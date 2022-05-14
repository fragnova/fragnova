use codec::{Decode, Encode};

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

// serde(rename_all = "camelCase") is needed or polkadot.js will not be able to deserialize

#[derive(Encode, Decode, Copy, Clone, PartialEq, Debug, Eq, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize), serde(rename_all = "camelCase"))]
pub enum AudioCategories {
	/// An audio file of the supported formats (mp3, ogg, wav, etc.)
	File,
	/// A chainblocks script that returns an effect chain that requires an input, validated
	Effect,
	/// A chainblocks script that returns an instrument chain (no audio input), validated
	Instrument,
}

#[derive(Encode, Decode, Copy, Clone, PartialEq, Debug, Eq, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize), serde(rename_all = "camelCase"))]
pub enum ModelCategories {
	/// A GLTF binary model
	Gltf,
	/// ???
	Sdf,
	/// A physics collision model
	PhysicsCollider,
}

#[derive(Encode, Decode, Copy, Clone, PartialEq, Debug, Eq, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize), serde(rename_all = "camelCase"))]
pub enum ShaderCategories {
	/// A chainblocks script that returns a shader chain (we validate that)
	Generic,
	/// A chainblocks script that returns a shader chain constrained to be a compute shader (we validate that)
	Compute,
	/// A chainblocks script that returns a shader chain constrained to be a screen post effect shader (we validate that)
	PostEffect,
}

#[derive(Encode, Decode, Copy, Clone, PartialEq, Debug, Eq, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize), serde(rename_all = "camelCase"))]
pub enum TextureCategories {
	PngFile,
	JpgFile,
}

#[derive(Encode, Decode, Copy, Clone, PartialEq, Debug, Eq, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize), serde(rename_all = "camelCase"))]
pub enum VectorCategories {
	SvgFile,
	FontFile,
}

#[derive(Encode, Decode, Copy, Clone, PartialEq, Debug, Eq, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize), serde(rename_all = "camelCase"))]
pub enum VideoCategories {
	Mp4File,
}

#[derive(Encode, Decode, Copy, Clone, PartialEq, Debug, Eq, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize), serde(rename_all = "camelCase"))]
pub enum TextCategories {
	Plain,
	Json,
}

#[derive(Encode, Decode, Copy, Clone, PartialEq, Debug, Eq, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize), serde(rename_all = "camelCase"))]
pub enum BinaryCategories {
	WasmModule,
}

#[derive(Encode, Decode, Copy, Clone, PartialEq, Debug, Eq, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize), serde(rename_all = "camelCase"))]
pub enum ChainCategories {
	/// A chainblocks script that returns a generic chain (we validate that)
	Generic,
	/// An animation sequence in chainblocks edn
	Animation,
	/// A chainblocks script that returns a chain constrained to be used as particle fx (we validate that)
	Particle,
}

/// Types of categories that can be attached to a Proto-Fragment to describe it (e.g Code, Audio, Video etc.)
#[derive(Encode, Decode, Copy, Clone, PartialEq, Debug, Eq, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize), serde(rename_all = "camelCase"))]
pub enum Categories {
	/// Chainblocks chains of various sub-categories
	Chain(ChainCategories),
	/// Audio files and effects
	Audio(AudioCategories),
	/// Textures of the supported sub-categories
	Texture(TextureCategories),
	/// Vectors of the supported sub-categories (e.g. SVG, Font)
	Vector(VectorCategories),
	/// Video file of the supported formats
	Video(VideoCategories),
	/// 2d/3d models of the supported formats
	Model(ModelCategories),
	/// A chainblocks script that returns a shader chain (we validate that)
	Shader(ShaderCategories),
	/// Text of the supported sub-categories
	Text(TextCategories),
	/// Binary of the supported sub-categories
	Binary(BinaryCategories),
}
