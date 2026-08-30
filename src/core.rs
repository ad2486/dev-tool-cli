use std::collections::HashMap;

use crate::providers::{ Provider };

pub struct Registry {
    providers: HashMap<String, Box<dyn Provider>>
}

impl Registry {
    fn new() -> Self {
        Self { providers: HashMap::new() }
    } 
    fn register(&mut self, provider: Box<dyn Provider>) {
        let name = provider.name().to_string();
        self.providers.insert(name, provider);
    }
    fn get(&self, name: &str) -> Option<&dyn Provider> {
        self.providers.get(name)
        .map(|provider| provider.as_ref())
    }
}