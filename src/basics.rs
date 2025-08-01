use super::*;

pub fn init_logging()
{
    #[cfg(not(target_arch = "wasm32"))]
    {
        #[cfg(target_os = "linux")]
        simple_logger::SimpleLogger::new().with_utc_timestamps().env().init().unwrap();
        #[cfg(not(target_os = "linux"))]
        simple_logger::SimpleLogger::new().with_local_timestamps().env().init().unwrap();
    }
    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init().unwrap();
    }
}

pub fn create_window(event_loop: &ActiveEventLoop) -> Window
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

#[cfg(not(target_arch = "wasm32"))]
pub mod time
{
    #[derive(Clone, Copy)]
    pub struct Instant(std::time::Instant);
    pub fn now() -> Instant { Instant(std::time::Instant::now()) }
    pub fn duration_secs(first: Instant, second: Instant) -> f32 { (second.0 - first.0).as_secs_f32() }
}

#[cfg(target_arch = "wasm32")]
pub mod time
{
    #[derive(Clone, Copy)]
    pub struct Instant(f64);
    pub fn now() -> Instant { Instant(web_sys::window().unwrap().performance().unwrap().now()) }
    pub fn duration_secs(first: Instant, second: Instant) -> f32 { ((second.0 - first.0) / 1e3) as f32 }
}
