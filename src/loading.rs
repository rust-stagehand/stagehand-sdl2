use sdl2::{
    image::LoadTexture,
    mixer::{Chunk, Music},
    render::{Texture, TextureCreator},
    ttf::{Font, Sdl2TtfContext},
    video::WindowContext,
};
use std::{marker::PhantomData, str, string::ToString};

use stagehand::{
    loading::{
        resources::{ResourceLoadError, ResourceLoader, ResourceStorage},
        ResourceError, Ticket, TicketManager,
    },
    utility::StorageType,
};

type TextureStorage<'a> =
    ResourceStorage<'a, String, Texture<'a>, TextureLoader<'a, WindowContext>>;
type SoundStorage<'a> = ResourceStorage<'a, String, Chunk, EmptyLoader>;
type MusicStorage<'a> = ResourceStorage<'a, String, Music<'a>, EmptyLoader>;
type FontStorage<'a, 'b, 'c> = ResourceStorage<'a, String, Font<'a, 'b>, FontLoader<'a, 'c>>;

pub struct SDLStorage<'a, 'b, 'c> {
    pub fonts: FontStorage<'a, 'b, 'c>,
    pub textures: TextureStorage<'a>,
    pub sounds: SoundStorage<'a>,
    pub music: MusicStorage<'a>,
}

impl<'a, 'b, 'c> SDLStorage<'a, 'b, 'c> {
    pub fn new(texture: &'a TextureLoader<WindowContext>, font: &'a FontLoader<'a, 'c>) -> Self {
        SDLStorage {
            fonts: FontStorage::new(font),
            textures: TextureStorage::new(texture),
            sounds: SoundStorage::new(&EmptyLoader {}),
            music: MusicStorage::new(&EmptyLoader {}),
        }
    }
}

impl<'a, 'b, 'c> TicketManager<StorageType, StorageType, String, str> for SDLStorage<'a, 'b, 'c> {
    fn get_ticket_with_key(
        &self,
        storage_key: &StorageType,
        resource_key: &str,
    ) -> Result<Ticket, ResourceError> {
        match storage_key {
            StorageType::Texture => self.textures.take_ticket(resource_key),
            StorageType::Font => self.fonts.take_ticket(resource_key),
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

pub struct FontLoader<'a, 'c> {
    pub context: Sdl2TtfContext,
    phantom: PhantomData<(&'a (), &'c ())>,
}

impl<'a, 'c> FontLoader<'a, 'c> {
    pub fn from_context(context: Sdl2TtfContext) -> Self {
        FontLoader {
            context,
            phantom: PhantomData,
        }
    }
}

impl<'a, 'b, 'c> ResourceLoader<'a, Font<'a, 'b>> for FontLoader<'a, 'c> {
    type Arguments = (&'c str, u16);

    fn load(&'a self, args: &Self::Arguments) -> Result<Font<'a, 'b>, ResourceLoadError> {
        let result = self.context.load_font(args.0, args.1);
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
