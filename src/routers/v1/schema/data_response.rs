use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DataResponse<T> {
    data: T,
    valid: bool
}


impl<T> DataResponse<T> {
    pub fn new(data: T) -> Self {
        Self { valid: true, data }
    }

    pub fn get_data(self) -> T {
        self.data
    }

    pub fn get_data_ref(&self) -> &T {
        &self.data
    }

    pub fn get_data_mut(&mut self) -> &mut T {
        &mut self.data
    }
}
