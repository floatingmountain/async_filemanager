use crate::{ AsyncFileManager, FileLoadFuture, LoadStatus};
use futures::executor::ThreadPool;
use std::{
    any::{type_name, Any, TypeId},
    collections::HashMap,
    sync::Arc,
};
use std::{
    convert::TryFrom,
    path::{Path, PathBuf},
};
pub trait Ron {
}

struct RonManager {
    pool: Arc<ThreadPool>,
    managers: HashMap<TypeId, Box<dyn Any>>,
}

impl RonManager {
    #[allow(unused)]
    fn new(pool: Arc<ThreadPool>) -> Self {
        Self {
            pool,
            managers: HashMap::new(),
        }
    }
    #[allow(unused)]
    fn register_material<T: Any + Ron + TryFrom<(PathBuf, Vec<u8>)> + Unpin>(&mut self) {
        let tid = TypeId::of::<T>();
        assert!(
            !self.managers.contains_key(&tid),
            "Material [{:?}] already registered!",
            type_name::<T>()
        );
        self.managers
            .entry(tid)
            .or_insert(Box::new(AsyncFileManager::<T>::new(self.pool.clone())));
    }
    #[allow(unused)]
    async fn load<T: Any + Ron + TryFrom<(PathBuf, Vec<u8>)> + Unpin, P: AsRef<Path>>(
        &mut self,
        path: P,
    ) {
        let tid = TypeId::of::<T>();
        if let Some(manager) = self
            .managers
            .get_mut(&tid)
            .and_then(|get| get.downcast_mut::<AsyncFileManager<T>>())
        {
            manager.load(path).await
        } else {
            panic!("Material [{:?}] not registered!", type_name::<T>());
        }
    }
    #[allow(unused)]
    async fn get<T: Any + Ron + TryFrom<(PathBuf, Vec<u8>)> + Unpin, P: AsRef<Path>>(
        &mut self,
        path: P,
    ) -> LoadStatus<T, FileLoadFuture<T>> {
        let tid = TypeId::of::<T>();
        if let Some(manager) = self
            .managers
            .get_mut(&tid)
            .and_then(|get| get.downcast_mut::<AsyncFileManager<T>>())
        {
            manager.get(path).await
        } else {
            panic!("Material [{:?}] not registered!", type_name::<T>());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Ron, RonManager};
    use crate::{LoadStatus};
    use futures::executor::ThreadPool;
    use std::{convert::TryFrom, path::PathBuf, sync::Arc};

    #[allow(unused)]
    struct Test {
        bytes: Vec<u8>,
    }
    impl TryFrom<(PathBuf, Vec<u8>)> for Test {
        type Error = std::io::Error;
        fn try_from((_, bytes): (PathBuf, Vec<u8>)) -> Result<Self, Self::Error> {
            Ok(Test { bytes })
        }
    }
    impl Ron for Test {
    }

    #[test]
    fn mattest() {
        let pool = Arc::new(ThreadPool::new().unwrap());
        let mut matman = RonManager::new(pool);
        matman.register_material::<Test>();
        let path = PathBuf::new().join("small_scream.png");
        futures::executor::block_on(async {
            matman.load::<Test, _>(&path).await;

            let _ = match matman.get::<Test, _>(&path).await {
                LoadStatus::Loaded(f) => f,
                LoadStatus::Loading(f) => f.await.unwrap(),
                _ => panic!(),
            };
        });
    }
}
