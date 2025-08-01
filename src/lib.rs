#![feature(let_chains)]

#[cfg(target_arch = "wasm32")]
pub use wasm_bindgen;
pub use winit;
pub use wgpu;
#[cfg(feature = "ui")]
pub use gru_ui as ui;
#[cfg(feature = "audio")]
pub use rodio;

mod basics;
pub use basics::time;
pub mod input;
pub mod graphics;
#[cfg(feature = "ui")]
pub mod ui_render;
#[cfg(feature = "storage")]
pub mod storage;
#[cfg(feature = "file")]
pub mod file;

use std::sync::Arc;
use winit::{application::ApplicationHandler, event::{WindowEvent, StartCause}, event_loop::{EventLoop, ActiveEventLoop, EventLoopProxy}, window::Window};

pub trait App: Sized + 'static
{
    const BACKENDS: wgpu::Backends;
    #[cfg(feature = "ui")]
    const DEPTH_FORMAT: Option<wgpu::TextureFormat>;
    type Init;
    #[cfg(feature = "ui")]
    type UiEvent;
    #[cfg(feature = "ui")]
    fn ui() -> gru_ui::Ui<'static, Self, Self::UiEvent>;
    fn init(init: Self::Init, ctx: &mut Context<Self>) -> Self;
    fn frame(&mut self, ctx: &mut Context<Self>, dt: f32) -> bool;
    fn deinit(self, _: &mut Context<Self>) -> Option<Self::Init> { None }
}

pub struct Context<T: App>
{
    pub window: Arc<Window>,
    pub input: input::Input,
    pub graphics: graphics::Graphics,
    #[cfg(not(feature = "ui"))]
    _phantom: std::marker::PhantomData<T>,
    #[cfg(feature = "ui")]
    pub ui: gru_ui::Ui<'static, T, T::UiEvent>,
    #[cfg(feature = "ui")]
    pub ui_render: ui_render::RenderData,
    #[cfg(feature = "audio")]
    pub audio: Option<(rodio::OutputStream, rodio::OutputStreamHandle)>,
    #[cfg(feature = "storage")]
    pub storage: storage::Storage,
}

impl<T: App> Context<T>
{
    async fn init(backends: wgpu::Backends, window: Window) -> Self
    {
        let window = Arc::new(window);
        let mut graphics = graphics::Graphics::init(backends, window.clone()).await.unwrap();
        let size = window.inner_size().into();
        graphics.configure(size);
        let input = input::Input::new();
        #[cfg(feature = "ui")]
        let (ui, ui_render) = (T::ui(), ui_render::RenderData::new(&graphics, T::DEPTH_FORMAT));

        window.set_visible(true);
        Self
        {
            window,
            input,
            graphics,
            #[cfg(not(feature = "ui"))]
            _phantom: std::marker::PhantomData,
            #[cfg(feature = "ui")]
            ui,
            #[cfg(feature = "ui")]
            ui_render,
            #[cfg(feature = "audio")]
            audio: None,
            #[cfg(feature = "storage")]
            storage: storage::Storage::load(),
        }
    }

    #[cfg(feature = "audio")]
    pub fn audio(&self) -> Option<&rodio::OutputStreamHandle> { self.audio.as_ref().map(|audio| &audio.1) }
}

enum AppState<T: App>
{
    Init(Option<T::Init>), //Option for moving out of ref
    App(T),
    Deinit,
}

struct AppHandler<T: App>
{
    ctx: Option<Context<T>>,
    event_loop_proxy: EventLoopProxy<Context<T>>,
    app: AppState<T>,
    then: time::Instant,
}

impl<T: App> AppHandler<T>
{
    fn new(init: T::Init, event_loop: &EventLoop<Context<T>>) -> Self
    {
        let event_loop_proxy = event_loop.create_proxy();
        Self { ctx: None, event_loop_proxy, app: AppState::Init(Some(init)), then: time::now() }
    }
}

impl<T: App> ApplicationHandler<Context<T>> for AppHandler<T>
{
    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: StartCause)
    {
        //init window & graphics
        if matches!(cause, StartCause::Init)
        {
            let window = basics::create_window(event_loop);
            let proxy = self.event_loop_proxy.clone();
            let future = async move
            {
                let ctx = Context::init(T::BACKENDS, window).await;
                proxy.send_event(ctx).ok().unwrap();
            };
            #[cfg(not(target_arch = "wasm32"))]
            pollster::block_on(future);
            #[cfg(target_arch = "wasm32")]
            wasm_bindgen_futures::spawn_local(future);
        }
    }

    fn resumed(&mut self, _: &winit::event_loop::ActiveEventLoop) {}

    fn user_event(&mut self, _: &ActiveEventLoop, mut ctx: Context<T>)
    {
        let AppState::Init(init) = &mut self.app else { unreachable!() };
        let init = init.take().unwrap();
        let app = T::init(init, &mut ctx);
        self.app = AppState::App(app);
        self.ctx = Some(ctx);
    }

    fn device_event(&mut self, _: &ActiveEventLoop, _: winit::event::DeviceId, event: winit::event::DeviceEvent)
    {
        if let Some(ctx) = self.ctx.as_mut()
        {
            ctx.input.event(input::RawEvent::Device(event));
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: winit::window::WindowId, event: WindowEvent)
    {
        if let Some(ctx) = self.ctx.as_mut()
        {
            #[cfg(feature = "audio")]
            if ctx.audio.is_none() && matches!(event, WindowEvent::MouseInput { .. })
            {
                ctx.audio = Some(rodio::OutputStream::try_default().unwrap());
            }
            match event
            {
                WindowEvent::Resized(new_size) => 
                {
                    let width = new_size.width.max(1);
                    let height = new_size.height.max(1);
                    ctx.graphics.configure((width, height));
                },
                WindowEvent::RedrawRequested => //frame
                {
                    let now = time::now();
                    let dt = time::duration_secs(self.then, now);
                    self.then = now;
                    let AppState::App(app) = &mut self.app else { unreachable!() };
                    if app.frame(ctx, dt) { event_loop.exit(); }
                    ctx.input.clear();
                    ctx.window.request_redraw();
                },
                event => ctx.input.event(input::RawEvent::Window(event)),
            }
        }
    }

    fn exiting(&mut self, _: &ActiveEventLoop)
    {
        let mut ctx = self.ctx.take().unwrap();
        let mut app = AppState::Deinit;
        std::mem::swap(&mut self.app, &mut app);
        let AppState::App(app) = app else { unreachable!() };
        let init = app.deinit(&mut ctx);
        std::mem::drop(ctx);
        std::mem::drop(init);
    }
}

pub fn run<T: App>(init: T::Init)
{
    basics::init_logging();
    
    #[cfg(target_os = "linux")]
    let mut event_loop =
    {
        use winit::platform::x11::EventLoopBuilderExtX11;
        event_loop::EventLoopBuilder::new().with_user_event().with_x11().build().unwrap()
    };
    #[cfg(not(target_os = "linux"))]
    let mut event_loop = event_loop::EventLoop::new().with_user_event().unwrap();
    
    let mut app: AppHandler<T> = AppHandler::new(init, &event_loop);
    event_loop.run_app(&mut app).unwrap();
}
