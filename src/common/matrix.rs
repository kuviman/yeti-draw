use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Matrix<T> {
    size: Vec2<usize>,
    data: Vec<T>,
}

impl<T> Matrix<T> {
    pub fn size(&self) -> Vec2<usize> {
        self.size
    }
    pub fn filled_with(size: Vec2<usize>, value: T) -> Self
    where
        T: Clone,
    {
        Self {
            size,
            data: vec![value; size.x * size.y],
        }
    }
    pub fn as_slice(&self) -> &[T] {
        self.data.as_slice()
    }
}

impl<T> Index<Vec2<usize>> for Matrix<T> {
    type Output = T;
    fn index(&self, index: Vec2<usize>) -> &T {
        &self.data[index.x * self.size.y + index.y]
    }
}

impl<T> IndexMut<Vec2<usize>> for Matrix<T> {
    fn index_mut(&mut self, index: Vec2<usize>) -> &mut T {
        &mut self.data[index.x * self.size.y + index.y]
    }
}
