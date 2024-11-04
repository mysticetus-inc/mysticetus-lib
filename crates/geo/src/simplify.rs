pub mod vw_preserve;

pub trait Simplify<Geom> {
    type Options;

    type Indices;

    type Error: std::error::Error + Send + 'static;

    fn new(opts: Self::Options) -> Self;

    fn simplify_geometry(&mut self, geom: &Geom) -> Result<Self::Indices, Self::Error>;
}
