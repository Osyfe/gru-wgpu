pub struct File
{
    #[cfg(not(target_arch = "wasm32"))]
    recv: flume::Receiver<anyhow::Result<Vec<u8>>>,
    #[cfg(target_arch = "wasm32")]
    request: (web_sys::XmlHttpRequest, bool),
}

impl File
{
    pub fn query(&mut self) -> Option<anyhow::Result<Vec<u8>>>
    {
        #[cfg(not(target_arch = "wasm32"))]
        return match self.recv.try_recv()
        {
            Ok(data) => Some(data),
            Err(flume::TryRecvError::Disconnected) => Some(Err(anyhow::anyhow!("Loader thread cancelled"))),
            Err(flume::TryRecvError::Empty) => None,
        };

        #[cfg(target_arch = "wasm32")]
        return if self.request.1 || self.request.0.ready_state() != 4 { None } //DONE
        else
        {
            self.request.1 = true;
            let status = self.request.0.status().unwrap();
            if status == 200 //OK
            {
                Some(Ok(js_sys::Uint8Array::new_with_byte_offset(&self.request.0.response().unwrap(), 0).to_vec()))
            } else
            {
                Some(Err(anyhow::anyhow!("Loading Status not OK")))
            }
        }
    }
}

pub struct Loader
{
    #[cfg(not(target_arch = "wasm32"))]
    thread: flume::Sender<(String, flume::Sender<anyhow::Result<Vec<u8>>>)>,
}

impl Loader
{
    pub fn new() -> Self
    {
        Self
        {
            #[cfg(not(target_arch = "wasm32"))]
            thread:
            {
                let (send, recv) = flume::unbounded::<(_, flume::Sender<_>)>();
                std::thread::spawn(move ||
                {
                    for (path, data_send) in recv
                    {
                        let data = std::fs::read(path).map_err(|err| anyhow::anyhow!("{err:?}"));
                        data_send.send(data).unwrap();
                    }
                });
                send
            },
        }
    }

    pub fn load(&mut self, path: &str) -> File
    {
        File
        {
            #[cfg(not(target_arch = "wasm32"))]
            recv:
            {
                let (send, recv) = flume::bounded(1);
                self.thread.send((path.to_owned(), send)).unwrap();
                recv
            },
            #[cfg(target_arch = "wasm32")]
            request:
            {
                let request = web_sys::XmlHttpRequest::new().unwrap();
                request.open_with_async("GET", path, true).unwrap();
                request.set_response_type(web_sys::XmlHttpRequestResponseType::Arraybuffer);
                request.send().unwrap();
                (request, false)
            },
        }
    }
}

impl std::future::Future for File
{
    type Output = anyhow::Result<Vec<u8>>;

    fn poll(mut self: std::pin::Pin<&mut Self>, _: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output>
    {
        match self.query()
        {
            Some(val) => std::task::Poll::Ready(val),
            None => std::task::Poll::Pending
        }
    }
}
