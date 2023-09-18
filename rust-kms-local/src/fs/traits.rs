pub trait Wrapper: Sized {
    type Error;
    type Args;

    fn load(options: Self::Args) -> Result<Self, Self::Error>;

    fn save(&self) -> Result<(), Self::Error>;
}
