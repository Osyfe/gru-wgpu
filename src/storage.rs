#[cfg(not(target_arch = "wasm32"))]
const PATH: &str = "CACHE.gru";

pub struct Storage
{
    #[cfg(not(target_arch = "wasm32"))]
    data: ahash::AHashMap<String, String>,
    #[cfg(target_arch = "wasm32")]
    data: web_sys::Storage,
}

impl Storage
{
    pub(crate) fn load() -> Self
    {
        Self
        {
            #[cfg(not(target_arch = "wasm32"))]
            data: std::fs::read(PATH).map(|contents| bincode::deserialize(&contents).unwrap()).unwrap_or_else(|_| ahash::AHashMap::new()),
            #[cfg(target_arch = "wasm32")]
            data: web_sys::window().unwrap().local_storage().unwrap().unwrap(),
        }
    }

    pub fn set(&mut self, key: &str, value: Option<&str>)
    {
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(value) = value { self.data.insert(key.to_string(), value.to_string()); }
        else { self.data.remove(key); }

        #[cfg(target_arch = "wasm32")]
        if let Some(value) = value { self.data.set_item(key, value).unwrap(); }
        else { self.data.remove_item(key).unwrap(); }
    }

    pub fn get(&self, key: &str) -> Option<String>
    {
        #[cfg(not(target_arch = "wasm32"))]
        return self.data.get(key).map(|value| value.to_string());

        #[cfg(target_arch = "wasm32")]
        return self.data.get_item(key).unwrap();
    }

    pub fn clear(&mut self)
    {
        #[cfg(not(target_arch = "wasm32"))]
        self.data.clear();

        #[cfg(target_arch = "wasm32")]
        self.data.clear().unwrap();
    }

    pub fn keys(&self) -> Vec<String>
    {
        #[cfg(not(target_arch = "wasm32"))]
        return self.data.keys().cloned().collect();

        #[cfg(target_arch = "wasm32")]
        return (0..self.data.length().unwrap()).map(|i| self.data.key(i).unwrap().unwrap()).collect();
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Drop for Storage
{
    fn drop(&mut self)
    {
        std::fs::write(PATH, bincode::serialize(&self.data).unwrap()).unwrap();
    }
}
