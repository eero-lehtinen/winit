use std::{fs::File, io::Write, sync::Arc};

use cursor_icon::CursorIcon;
use rustix::fd::{AsFd, OwnedFd};
use wayland_backend::client::ObjectData;
use wayland_client::{
    protocol::{
        wl_buffer::{self, WlBuffer},
        wl_shm::{self, Format, WlShm},
        wl_shm_pool::{self, WlShmPool},
    },
    Connection, Proxy, WEnum,
};

#[derive(Debug)]
pub struct WaylandCustomCursor {
    pub bgra_bytes: Vec<u8>,
    pub w: i32,
    pub h: i32,
    pub hot_x: i32,
    pub hot_y: i32,
}

#[derive(Debug)]
pub struct CustomCursorData {
    _file: File,
    shm_pool: WlShmPool,
    pub buffer: WlBuffer,
    pub w: i32,
    pub h: i32,
    pub hot_x: i32,
    pub hot_y: i32,
    pub original: Arc<WaylandCustomCursor>,
}

impl CustomCursorData {
    pub fn new(connection: &Connection, shm: &WlShm, cursor: Arc<WaylandCustomCursor>) -> Self {
        let mfd = memfd::MemfdOptions::default()
            .close_on_exec(true)
            .create("winit-custom-cursor")
            .unwrap();
        let mut file = mfd.into_file();
        file.set_len(cursor.bgra_bytes.len() as u64).unwrap();
        file.write_all(&cursor.bgra_bytes).unwrap();
        file.flush().unwrap();

        let pool_id = connection
            .send_request(
                shm,
                wl_shm::Request::CreatePool {
                    size: cursor.bgra_bytes.len() as i32,
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
                    width: cursor.w,
                    height: cursor.h,
                    stride: cursor.w * 4,
                    format: WEnum::Value(Format::Argb8888),
                },
                Some(Arc::new(IgnoreObjectData)),
            )
            .unwrap();
        let buffer = WlBuffer::from_id(connection, buffer_id).unwrap();

        CustomCursorData {
            _file: file,
            shm_pool,
            buffer,
            w: cursor.w,
            h: cursor.h,
            hot_x: cursor.hot_x,
            hot_y: cursor.hot_y,
            original: cursor,
        }
    }

    pub fn destroy(&self, connection: &Connection) {
        // I guess this should be called only after we get wl_buffer.release, but I don't know how
        connection
            .send_request(&self.buffer, wl_buffer::Request::Destroy, None)
            .unwrap();
        connection
            .send_request(&self.shm_pool, wl_shm_pool::Request::Destroy, None)
            .unwrap();
    }
}

#[derive(Debug)]
pub enum SelectedCursor {
    BuiltIn(CursorIcon),
    Custom(CustomCursorData),
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
