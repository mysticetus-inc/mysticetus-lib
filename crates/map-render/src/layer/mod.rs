use std::collections::BTreeMap;

use crate::feature::{Drawable, Feature};

pub struct Layers<'a, Id: Clone + Ord = usize> {
    by_layer: BTreeMap<Id, Layer<'a, Id>>,
}

impl<'a, Id: Ord + Clone> Layers<'a, Id> {
    pub fn get_layer(&mut self, layer: Id) -> &mut Layer<'a, Id> {
        self.by_layer.entry(layer).or_insert_with_key(|id| Layer {
            id: id.clone(),
            drawables: Vec::with_capacity(32),
        })
    }
}

pub struct Layer<'a, Id> {
    id: Id,
    drawables: Vec<Box<dyn Drawable + 'a>>,
}

impl<'a, Id> Layer<'a, Id> {
    pub fn id(&self) -> &Id {
        &self.id
    }

    pub fn add_feature<F: Feature + 'a>(&mut self, feature: F) -> &mut Self {
        self.drawables.push(Box::new(feature));
        self
    }

    pub fn add_boxed_drawable(&mut self, boxed_drawable: Box<dyn Drawable + 'a>) -> &mut Self {
        self.drawables.push(boxed_drawable);
        self
    }

    pub fn add_drawable<D: Drawable + 'a>(&mut self, drawable: D) -> &mut Self {
        self.add_boxed_drawable(Box::new(drawable))
    }

    pub fn add_drawables<I>(&mut self, drawables: I) -> &mut Self
    where
        I: IntoIterator,
        I::Item: Drawable + 'a,
    {
        fn into_boxed<'a, D: Drawable + 'a>(item: D) -> Box<dyn Drawable + 'a> {
            Box::new(item)
        }

        self.add_boxed_drawables(drawables.into_iter().map(into_boxed::<'a, I::Item>))
    }

    pub fn add_boxed_drawables<I>(&mut self, drawables: I) -> &mut Self
    where
        I: IntoIterator<Item = Box<dyn Drawable + 'a>>,
    {
        self.drawables.extend(drawables.into_iter());
        self
    }
}
