use std::{fs::File, io::Write, sync::Arc};

use cursor_icon::CursorIcon;
use rustix::fd::{AsFd, OwnedFd};
use wayland_backend::client::ObjectData;
use wayland_client::{
    protocol::{
        wl_buffer::WlBuffer,
        wl_shm::{self, Format, WlShm},
        wl_shm_pool::{self, WlShmPool},
    },
    Connection, Proxy, WEnum,
};

#[derive(Debug)]
pub struct CustomCursor {
    _file: File,
    pub buffer: WlBuffer,
    pub w: i32,
    pub h: i32,
    pub hot_x: i32,
    pub hot_y: i32,
}

impl CustomCursor {
    pub fn new(
        connection: &Connection,
        shm: &WlShm,
        bgra_bytes: &[u8],
        w: i32,
        h: i32,
        hot_x: i32,
        hot_y: i32,
    ) -> Self {
        let mfd = memfd::MemfdOptions::default()
            .close_on_exec(true)
            .create("winit-custom-cursor")
            .unwrap();
        let mut file = mfd.into_file();
        file.set_len(bgra_bytes.len() as u64).unwrap();
        file.write_all(bgra_bytes).unwrap();
        file.flush().unwrap();

        let pool_id = connection
            .send_request(
                shm,
                wl_shm::Request::CreatePool {
                    size: bgra_bytes.len() as i32,
                    fd: file.as_fd(),
                },
                Some(Arc::new(IgnoreObjectData)),
            )
            .unwrap();
        let shm_pool = WlShmPool::from_id(connection, pool_id).unwrap();

        let buffer_id = connection
            .send_request(
                &shm_pool,
                wl_shm_pool::Request::CreateBuffer {
                    offset: 0,
                    width: w,
                    height: h,
                    stride: (w * 4),
                    format: WEnum::Value(Format::Argb8888),
                },
                Some(Arc::new(IgnoreObjectData)),
            )
            .unwrap();
        let buffer = WlBuffer::from_id(connection, buffer_id).unwrap();

        CustomCursor {
            _file: file,
            buffer,
            w,
            h,
            hot_x,
            hot_y,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SelectedCursor {
    BuiltIn(CursorIcon),
    Custom(u64),
}

impl Default for SelectedCursor {
    fn default() -> Self {
        Self::BuiltIn(Default::default())
    }
}

struct IgnoreObjectData;

impl ObjectData for IgnoreObjectData {
    fn event(
        self: Arc<Self>,
        _: &wayland_client::backend::Backend,
        _: wayland_client::backend::protocol::Message<wayland_client::backend::ObjectId, OwnedFd>,
    ) -> Option<Arc<dyn ObjectData>> {
        None
    }
    fn destroyed(&self, _: wayland_client::backend::ObjectId) {}
}
