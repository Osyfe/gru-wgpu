#[cfg(target_arch = "wasm32")]
pub use wasm_bindgen;

pub use winit;
pub use wgpu;

pub mod input;
pub mod graphics;

use std::sync::Arc;
use winit::{application::ApplicationHandler, event::{WindowEvent, StartCause}, event_loop::{EventLoop, ActiveEventLoop, EventLoopProxy}, window::Window};

pub trait App
{
    const BACKENDS: wgpu::Backends;
    type Init;
    fn init(init: Self::Init) -> Self;
    fn frame(&mut self, ctx: &mut Context) -> bool;
    fn deinit(self, ctx: &mut Context) -> Option<Self::Init>;
}

#[derive(Debug)]
pub struct Context
{
    pub window: Arc<Window>,
    pub input: input::Input,
    pub graphics: graphics::Graphics,
}

impl Context
{
    async fn init(backends: wgpu::Backends, window: Window) -> Self
    {
        let window = Arc::new(window);
        let mut graphics = graphics::Graphics::init(backends, window.clone()).await.unwrap();
        let size = window.inner_size().into();
        graphics.configure(size);
        let input = input::Input::new();

        window.set_visible(true);
        Self { window, input, graphics }
    }
}

enum AppState<T: App>
{
    Init(Option<T::Init>), //Option for moving out of ref
    App(T),
    Deinit,
}

struct AppHandler<T: App>
{
    ctx: Option<Context>,
    event_loop_proxy: EventLoopProxy<Context>,
    app: AppState<T>,
}

impl<T: App> AppHandler<T>
{
    fn new(init: T::Init, event_loop: &EventLoop<Context>) -> Self
    {
        let event_loop_proxy = event_loop.create_proxy();
        Self { ctx: None, event_loop_proxy, app: AppState::Init(Some(init)) }
    }
}

impl<T: App> ApplicationHandler<Context> for AppHandler<T>
{
    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: StartCause)
    {
        //init window & graphics
        if matches!(cause, StartCause::Init)
        {
            let window = create_window(event_loop);
            let proxy = self.event_loop_proxy.clone();
            let future = async move
            {
                let ctx = Context::init(T::BACKENDS, window).await;
                proxy.send_event(ctx).unwrap();
            };
            #[cfg(not(target_arch = "wasm32"))]
            pollster::block_on(future);
            #[cfg(target_arch = "wasm32")]
            wasm_bindgen_futures::spawn_local(future);
        }
    }

    fn resumed(&mut self, _: &winit::event_loop::ActiveEventLoop) {}

    fn user_event(&mut self, _: &ActiveEventLoop, event: Context)
    {
        let AppState::Init(init) = &mut self.app else { unreachable!() };
        let init = init.take().unwrap();
        let app = T::init(init);
        self.app = AppState::App(app);
        self.ctx = Some(event);
    }

    fn device_event(&mut self, _: &ActiveEventLoop, _: winit::event::DeviceId, event: winit::event::DeviceEvent)
    {
        if let Some(ctx) = self.ctx.as_mut()
        {
            ctx.input.event(input::RawEvent::Device(event));
        }
    }

    fn window_event(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, _: winit::window::WindowId, event: WindowEvent)
    {
        if let Some(ctx) = self.ctx.as_mut()
        {
            match event
            {
                WindowEvent::Resized(new_size) => 
                {
                    let width = new_size.width.max(1);
                    let height = new_size.height.max(1);
                    ctx.graphics.configure((width, height));
                },
                WindowEvent::RedrawRequested =>
                {
                    match &mut self.app
                    {
                        AppState::App(app) => if app.frame(ctx) { event_loop.exit(); },
                        _ => unreachable!(),
                    }
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
    init_logging();
    let event_loop = EventLoop::with_user_event().build().unwrap();
    let mut app: AppHandler<T> = AppHandler::new(init, &event_loop);
    //event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.run_app(&mut app).unwrap();
}

fn init_logging()
{
    #[cfg(not(target_arch = "wasm32"))]
    {
        #[cfg(target_os = "linux")]
        simple_logger::SimpleLogger::new().with_utc_timestamps().init().unwrap();
        #[cfg(not(target_os = "linux"))]
        simple_logger::SimpleLogger::new().with_local_timestamps().init().unwrap();
    }
    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init().unwrap();
    }
}

fn create_window(event_loop: &ActiveEventLoop) -> Window
{
    #[cfg(not(target_arch = "wasm32"))]
    {
        #[allow(unused_mut)]
        let mut attribs = Window::default_attributes()
            .with_visible(false)
            .with_resizable(true);
        #[cfg(target_os = "windows")]
        {
            use winit::platform::windows::WindowAttributesExtWindows;
            attribs = attribs.with_drag_and_drop(false); //conflicts with cpal
        }
        event_loop.create_window(attribs).expect("Window creation failed.")
    }
    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::WindowAttributesExtWebSys;
        use wasm_bindgen::JsCast;
        let web_window = web_sys::window().unwrap();
        let canvas: web_sys::HtmlCanvasElement = web_window
            .document().unwrap()
            .get_element_by_id("canvas").unwrap()
            .dyn_into().unwrap();
        let attribs = Window::default_attributes().with_canvas(Some(canvas));
        event_loop.create_window(attribs).unwrap()
    }
}
