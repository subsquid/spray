pub type Predicate<T> = Box<dyn Fn(&T) -> bool + Send>;


pub struct ItemFilter<T, R> {
    predicates: Vec<Predicate<T>>,
    relations: R,
}


impl<T, R> ItemFilter<T, R> {
    pub fn eval(&self, val: &T) -> bool {
        for pred in self.predicates.iter() {
            if !pred(val) {
                return false
            }
        }
        true
    }
    
    #[allow(unused)]
    pub fn matches_all(&self) -> bool {
        self.predicates.is_empty()
    }

    pub fn relations(&self) -> &R {
        &self.relations
    }

    pub fn relations_mut(&mut self) -> &mut R {
        &mut self.relations
    }

    pub fn add<F>(&mut self, pred: F)
    where
        F: 'static + Fn(&T) -> bool + Send
    {
        self.predicates.push(Box::new(pred))
    }
}


pub trait Relations: Default {
    fn include(&mut self, other: &Self);
}


impl<T, R: Relations> ItemFilter<T, R> {
    pub fn or(filters: &[Self], val: &T) -> Option<R> {
        let mut relations = R::default();
        let mut pass = false;
        for f in filters {
            if f.eval(val) {
                pass = true;
                relations.include(f.relations());
            }
        }
        pass.then_some(relations)
    }
}


impl<T, R: Default> Default for ItemFilter<T, R> {
    fn default() -> Self {
        Self {
            predicates: Vec::new(),
            relations: R::default()
        }
    }
}