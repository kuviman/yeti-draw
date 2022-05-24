use super::*;

pub mod autosaved;
mod matrix;

pub use autosaved::AutoSaved;
pub use matrix::*;

pub fn div_down<T: Num>(a: T, b: T) -> T {
    if a < T::ZERO {
        return -div_up(-a, b);
    }
    if b < T::ZERO {
        return -div_up(a, -b);
    }
    a / b
}

pub fn div_up<T: Num>(a: T, b: T) -> T {
    if a < T::ZERO {
        return -div_down(-a, b);
    }
    if b < T::ZERO {
        return -div_down(a, -b);
    }
    (a + b - T::ONE) / b
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Pixel {
    pub position: Vec2<i32>,
    pub color: Color<u8>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Update {
    Draw(Vec<Pixel>),
}

pub type UpdateId = u64;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ClientMessage {
    Download { area: AABB<i32> },
    Update { id: UpdateId, update: Update },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ServerMessage {
    Download {
        position: Vec2<i32>,
        data: Matrix<Color<u8>>,
    },
    Update {
        your_id: Option<UpdateId>,
        update: Update,
    },
}
