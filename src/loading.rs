use sdl2::{
    image::LoadTexture,
    mixer::{Chunk, Music},
    render::{Texture, TextureCreator},
    video::WindowContext,
};
use std::{marker::PhantomData, str, string::ToString};

use stagehand::{
    loading::{
        resources::{ResourceLoadError, ResourceLoader, ResourceStorage},
        ResourceError, Ticket, TicketManager,
    },
    utility2d::StorageType,
};

type TextureStorage<'a> =
    ResourceStorage<'a, String, Texture<'a>, TextureLoader<'a, WindowContext>>;
type SoundStorage<'a> = ResourceStorage<'a, String, Chunk, EmptyLoader>;
type MusicStorage<'a> = ResourceStorage<'a, String, Music<'a>, EmptyLoader>;

pub struct SDLStorage<'a> {
    pub textures: TextureStorage<'a>,
    pub sounds: SoundStorage<'a>,
    pub music: MusicStorage<'a>,
}

impl<'a> SDLStorage<'a> {
    pub fn new(texture: &'a TextureLoader<WindowContext>) -> Self {
        SDLStorage {
            textures: TextureStorage::new(texture),
            sounds: SoundStorage::new(&EmptyLoader {}),
            music: MusicStorage::new(&EmptyLoader {}),
        }
    }
}

impl<'a> TicketManager<StorageType, StorageType, String, str> for SDLStorage<'a> {
    fn get_ticket_with_key(
        &self,
        storage_key: &StorageType,
        resource_key: &str,
    ) -> Result<Ticket, ResourceError> {
        match storage_key {
            StorageType::Texture => self.textures.take_ticket(resource_key),
            StorageType::Music => self.music.take_ticket(resource_key),
            StorageType::Sound => self.sounds.take_ticket(resource_key),
            _ => Err(ResourceError::UnknownStorage(storage_key.to_string())),
        }
    }
}

pub struct TextureLoader<'a, T> {
    pub creator: TextureCreator<T>,
    phantom: PhantomData<&'a ()>,
}

impl<'a, T> TextureLoader<'a, T> {
    pub fn from_creator(creator: TextureCreator<T>) -> Self {
        TextureLoader {
            creator,
            phantom: PhantomData,
        }
    }
}

impl<'a, T> ResourceLoader<'a, Texture<'a>> for TextureLoader<'a, T> {
    type Arguments = str;

    fn load(&'a self, args: &Self::Arguments) -> Result<Texture<'a>, ResourceLoadError> {
        let result = self.creator.load_texture(args);
        match result {
            Ok(t) => Ok(t),
            Err(e) => Err(ResourceLoadError::LoadFailure(e)),
        }
    }
}

pub struct EmptyLoader {}

impl<'a> ResourceLoader<'a, Music<'a>> for EmptyLoader {
    type Arguments = str;

    fn load(&'a self, args: &Self::Arguments) -> Result<Music<'a>, ResourceLoadError> {
        match sdl2::mixer::Music::from_file(args) {
            Ok(m) => Ok(m),
            Err(e) => Err(ResourceLoadError::LoadFailure(e)),
        }
    }
}

impl<'a> ResourceLoader<'a, Chunk> for EmptyLoader {
    type Arguments = str;

    fn load(&'a self, args: &Self::Arguments) -> Result<Chunk, ResourceLoadError> {
        match sdl2::mixer::Chunk::from_file(args) {
            Ok(c) => Ok(c),
            Err(e) => Err(ResourceLoadError::LoadFailure(e)),
        }
    }
}
