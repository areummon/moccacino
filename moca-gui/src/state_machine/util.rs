pub trait VectorExt {
    fn length(&self) -> f32;
    fn unit(&self) -> Self;
}

impl VectorExt for iced::Vector {
    fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    fn unit(&self) -> Self {
        let length = self.length();
        if length > 1e-6 {
            iced::Vector::new(self.x / length, self.y / length)
        } else {
            iced::Vector::new(0.0, 0.0)
        }
    }
}
