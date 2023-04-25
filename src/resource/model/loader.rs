//! Model loader.

use crate::{
    asset::{
        loader::{BoxedLoaderFuture, ResourceLoader},
        manager::ResourceManager,
        options::try_get_import_settings,
    },
    engine::SerializationContext,
    resource::model::{Model, ModelImportOptions},
    utils::log::Log,
};
use fyrox_resource::event::ResourceEventBroadcaster;
use fyrox_resource::untyped::UntypedResource;
use std::any::Any;
use std::sync::Arc;

/// Default implementation for model loading.
pub struct ModelLoader {
    /// Resource manager to allow complex model loading.
    pub resource_manager: ResourceManager,
    /// Node constructors contains a set of constructors that allows to build a node using its
    /// type UUID.
    pub serialization_context: Arc<SerializationContext>,
    /// Default import options for model resources.
    pub default_import_options: ModelImportOptions,
}

impl ResourceLoader for ModelLoader {
    fn extensions(&self) -> &[&str] {
        &["rgs", "fbx"]
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn load(
        &self,
        model: UntypedResource,
        event_broadcaster: ResourceEventBroadcaster,
        reload: bool,
    ) -> BoxedLoaderFuture {
        let resource_manager = self.resource_manager.clone();
        let node_constructors = self.serialization_context.clone();
        let default_import_options = self.default_import_options.clone();

        Box::pin(async move {
            let path = model.path().to_path_buf();

            let import_options = try_get_import_settings(&path)
                .await
                .unwrap_or(default_import_options);

            match Model::load(&path, node_constructors, resource_manager, import_options).await {
                Ok(raw_model) => {
                    Log::info(format!("Model {:?} is loaded!", path));

                    model.0.lock().commit_ok(raw_model);

                    event_broadcaster.broadcast_loaded_or_reloaded(model, reload);
                }
                Err(error) => {
                    Log::err(format!(
                        "Unable to load model from {:?}! Reason {:?}",
                        path, error
                    ));

                    model.0.lock().commit_error(path, error);
                }
            }
        })
    }
}