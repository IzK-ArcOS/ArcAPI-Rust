use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DataResponse<T> {
    pub data: T,
    pub valid: bool,
    #[serde(rename = "statusCode")]
    pub status_code: i32
}


impl<T> DataResponse<T> {
    pub fn new(data: T) -> Self {
        Self { valid: true, status_code: 200, data }
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


#[derive(Debug, Serialize, Deserialize)]
pub struct FlatDataResponse<T> {
    #[serde(flatten)]
    data: T,
    valid: bool,
    #[serde(rename = "statusCode")]
    status_code: i32
}


impl<T> FlatDataResponse<T> {
    pub fn new(data: T) -> Self {
        Self { valid: true, status_code: 200, data }
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
