//! File and filesystem-related syscalls
use crate::drivers::BLOCK_DEVICE;
use crate::fs::ROOT_INODE;
use easy_fs::{block_cache_sync_all, DirEntry, DIRENT_SZ};
use crate::fs::{open_file, OpenFlags, Stat};
use crate::mm::{translated_byte_buffer, translated_str, UserBuffer};
use crate::task::{current_task, current_user_token};

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    trace!("kernel:pid[{}] sys_write", current_task().unwrap().pid.0);
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        if !file.writable() {
            return -1;
        }
        let file = file.clone();
        // release current task TCB manually to avoid multi-borrow
        drop(inner);
        file.write(UserBuffer::new(translated_byte_buffer(token, buf, len))) as isize
    } else {
        -1
    }
}

pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    trace!("kernel:pid[{}] sys_read", current_task().unwrap().pid.0);
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        let file = file.clone();
        if !file.readable() {
            return -1;
        }
        // release current task TCB manually to avoid multi-borrow
        drop(inner);
        trace!("kernel: sys_read .. file.read");
        file.read(UserBuffer::new(translated_byte_buffer(token, buf, len))) as isize
    } else {
        -1
    }
}

pub fn sys_open(path: *const u8, flags: u32) -> isize {
    trace!("kernel:pid[{}] sys_open", current_task().unwrap().pid.0);
    let task = current_task().unwrap();
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(inode) = open_file(path.as_str(), OpenFlags::from_bits(flags).unwrap()) {
        let mut inner = task.inner_exclusive_access();
        let fd = inner.alloc_fd();
        inner.fd_table[fd] = Some(inode);
        fd as isize
    } else {
        -1
    }
}

pub fn sys_close(fd: usize) -> isize {
    trace!("kernel:pid[{}] sys_close", current_task().unwrap().pid.0);
    let task = current_task().unwrap();
    let mut inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if inner.fd_table[fd].is_none() {
        return -1;
    }
    inner.fd_table[fd].take();
    0
}

/// YOUR JOB: Implement fstat.
pub fn sys_fstat(_fd: usize, _st: *mut Stat) -> isize {
    trace!(
        "kernel:pid[{}] sys_fstat",
        current_task().unwrap().pid.0
    );


    let task = current_task().unwrap();
    let token = current_user_token();
    let inner = task.inner_exclusive_access();

    if _fd >= inner.fd_table.len() {
        return -1;
    }

    let file = match &inner.fd_table[_fd] {
        Some(f) => f.clone(),
        None => return -1,
    };

    drop(inner);

    let stat = file.stat();

    let stat_size = core::mem::size_of::<Stat>();


    let v = translated_byte_buffer(token, _st as *const u8, stat_size);
    
    let stat_bytes = unsafe {core::slice::from_raw_parts(
        &stat as *const Stat as *const u8, 
        stat_size
    )};


    let mut offset = 0;
    for page in v {
        let len = page.len().min(stat_bytes.len() - offset);
        page[..len].copy_from_slice(&stat_bytes[offset..offset + len]);
        offset += len;

        if offset >= stat_bytes.len() {
            break;
        }
    }

    // println!("fstat success !!!");

    0
    
}

/// YOUR JOB: Implement linkat.
pub fn sys_linkat(_old_name: *const u8, _new_name: *const u8) -> isize {
    trace!(
        "kernel:pid[{}] sys_linkat",
        current_task().unwrap().pid.0
    );


    let token = current_user_token();
    let old_path = translated_str(token, _old_name);
    let new_path = translated_str(token, _new_name);

    if old_path == new_path {
        println!("Error: old_path and new_path are the same");
        return -1;
    }

    let old_inode = ROOT_INODE.find(&old_path.as_str()).unwrap();

    let fs = old_inode.get_fs();
    let mut fs_lock = fs.lock();


    let old_inode_id = old_inode.get_inode_id(fs_lock.get_inode_start_block());

    ROOT_INODE.modify_disk_inode(|root_inode| {
        let file_count = (root_inode.size as usize) / DIRENT_SZ;

        let new_size = (file_count + 1) * DIRENT_SZ;
        ROOT_INODE.increase_size(new_size as u32, root_inode, &mut fs_lock);
        let dirent = DirEntry::new(new_path.as_str(), old_inode_id);
        root_inode.write_at(file_count * DIRENT_SZ, dirent.as_bytes(), &BLOCK_DEVICE);

    });

    old_inode.increment_nlink();


    block_cache_sync_all();

    0
}

/// Implement unlinkat.
pub fn sys_unlinkat(_name: *const u8) -> isize {
    trace!(
        "kernel:pid[{}] sys_unlinkat",
        current_task().unwrap().pid.0
    );

    let token = current_user_token();

    let path = translated_str(token, _name);

    let inode = ROOT_INODE.find(&path.as_str()).unwrap();

    let mut found = false;

    ROOT_INODE.modify_disk_inode(|root_inode| {

        let file_count = (root_inode.size as usize) / DIRENT_SZ;

        let mut dirent = DirEntry::empty();

        for i in 0..file_count {
            root_inode.read_at(DIRENT_SZ * i, dirent.as_bytes_mut(), &BLOCK_DEVICE);
            if dirent.name() == path.as_str() {
                found = true;
                if i != file_count - 1 {
                    let mut last_dirent = DirEntry::empty();
                    root_inode.read_at(DIRENT_SZ * (file_count - 1), last_dirent.as_bytes_mut(), &BLOCK_DEVICE);
                    root_inode.write_at(DIRENT_SZ * i, last_dirent.as_bytes(), &BLOCK_DEVICE);
                }
                root_inode.size -= DIRENT_SZ as u32;
                break;
            }
        }
    });

    if !found {
        return -1;
    }
    inode.decrement_nlink();

    if inode.get_nlink() == 0 {
        inode.clear();
    }
    block_cache_sync_all();
    
    0
}
