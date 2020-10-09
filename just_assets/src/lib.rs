use just_core::{
    traits::scripting::ScriptApiRegistry,
    ecs::prelude::*
};
use std::any::TypeId;
use std::collections::HashMap;
use std::path::PathBuf;

pub struct AssetManager {
    files_to_load: HashMap<String, Vec<(PathBuf, Vec<u8>)>>,
}

pub struct AssetSystem;

impl AssetSystem {
    pub fn initialize(world: &mut World, resources: &str) {
        println!("initializing resource system");
        println!(
            "Asset system: Loading resources from: {}",
            std::fs::canonicalize(resources).unwrap().display()
        );
        let mut files_to_load = HashMap::new();

        std::fs::read_dir(resources)
            .map_err(|_| "counldn't read directory")
            .unwrap().for_each(|x| {
                let path = x.unwrap().path();
                let extension = path.extension().unwrap().to_str();
                if let None = extension {
                    return;
                }
                let extension = extension.unwrap().to_owned();
                let data = std::fs::read(path.clone()).unwrap();
                files_to_load.entry(extension).or_insert_with(|| Vec::new()).push((path, data));
            });


        for pair in &files_to_load {
            println!("type: {}", pair.0);
            for file in pair.1 {
                println!("\tfile: {:?}", file.0);

            }
        }

        world.resources.insert::<AssetManager>(AssetManager {
            files_to_load,
        });
    }

    pub fn register_api<SAR: ScriptApiRegistry>(sar: &mut SAR) {

    }

    pub fn update(world: &mut World) {

    }

    pub fn cleanup(world: &mut World) {

    }

}

impl AssetManager {
    fn process_extension<F: FnMut(&std::path::PathBuf, &[u8])->bool>(&mut self, ext: &str, mut fun: F) {
        match self.files_to_load.get_mut(ext) {
            None => (),
            Some(x) => {
                for file in x.drain(0..x.len()) {
                    fun(&file.0, &file.1);
                }

            }
        }
    }

    fn get_entries(&self, ext: &str) -> Option<impl std::iter::Iterator<Item=std::path::PathBuf> + '_> {
        if let Some(ref x) = self.files_to_load.get(ext) {
            return Some(x.iter().map(|x| x.0.clone()));
        }
        return None;
    }
}

//#[derive(Copy, Clone)]
pub struct Handle<A> {
    _phantom: std::marker::PhantomData<A>,
    id: usize,
}

impl<T> Copy for Handle<T> {}
impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Handle{
            _phantom: Default::default(),
            id: self.id,
        }
    }
}

impl<T> std::fmt::Debug for Handle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.id.fmt(f)
    }
}

impl<T> std::cmp::PartialEq for Handle<T> {
    fn eq(&self, rhs: &Self) -> bool {
        self.id == rhs.id
    }

}

pub struct AssetStorage<A> {
    names: HashMap<String, Handle<A>>,
    assets: HashMap<usize, Asset<A>>,
    last_id: usize,
}

pub enum AssetState<A> {
    Offline,
    Queued,
    Loaded(A),
}

impl<T> std::fmt::Debug for AssetState<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Offline => "Offline",
            Self::Queued => "Queued",
            Self::Loaded(..) => "Loaded",
        }.fmt(f)
    }
}

pub struct Asset<A> {
    state: AssetState<A>,
}

impl<T> std::fmt::Debug for Asset<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Asset").field("state", &self.state).finish()
    }
}

impl<T> AssetStorage<T> {
    pub fn empty(manager: &AssetManager, exts: &[&str]) -> Self {
        let mut last_id = 1;
        let mut names = HashMap::new();
        let mut assets = HashMap::new();
        for ext in exts {
            match manager.get_entries(ext){
                None=> continue,
                Some(x) =>{
                    for file in x {
                        let name = file.file_stem().unwrap().to_str().unwrap();
                        println!("adding empty entryfile: {:?} with name: {}", file, name);
                        let id = last_id;
                        last_id+=1;
                        names.insert(name.to_owned(), Handle {id, _phantom: Default::default()});
                        assets.insert(id, Asset{state: AssetState::Offline});
                        
                    }
                }
            }
        }

        Self {
            names,
            assets,
            last_id,
        }
    }

    pub fn process<F: FnMut(&[u8])->(T, bool)>(&mut self, manager: &mut AssetManager, ext: &str, mut p: F) {
        manager.process_extension(ext, |name, data| {
            println!("processing {} file: {:?}", ext, name);
            let result = p(data);
            let name = name.file_stem().unwrap().to_str().unwrap();
            if self.names.contains_key(name) {
                let id = self.names[name];
                self.assets.get_mut(&id.id).unwrap().state = AssetState::Loaded(result.0);
            }
            else {
                let id = self.last_id+1;
                self.last_id+=1;
                self.names.insert(name.to_owned(), Handle{id, _phantom: Default::default()});
                self.assets.insert(id, Asset{state: AssetState::Loaded(result.0)});
            }
            println!("all assets: {:#?}", self.names);
            println!("all assets data: {:#?}", self.assets);
            result.1
        });
    }

    pub fn get_handle(&self, name: &str) -> Option<Handle<T>> {
        self.names.get(name).copied()

    }

    pub fn get_value(&self, handle: &Handle<T>) -> Option<&T> {
        match self.assets.get(&handle.id) {
            None => {
                println!("handle {} was requested but not present", handle.id);
                None
            },
            Some(x) => {
                match &x.state {
                    AssetState::Loaded(val) => Some(val),
                    _ => {
                        println!("handle {} was in not loaded state", handle.id);
                        None
                    },
                }
            }
        }
    }

}

