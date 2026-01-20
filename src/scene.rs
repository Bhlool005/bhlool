use crate::shapes::Rectangle;

#[derive(Debug, Default)]
pub struct Scene {
    rectangles: Vec<Rectangle>,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            rectangles: Vec::new(),
        }
    }

    pub fn add_rectangle(&mut self, rectangle: Rectangle) {
        self.rectangles.push(rectangle);
    }

    pub fn rectangles(&self) -> &[Rectangle] {
        &self.rectangles
    }
}
