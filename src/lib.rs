use std::ffi::{c_char, c_int, c_uchar, c_void, CStr};
use std::marker::PhantomData;

use libc::free;

pub type xmlChar = c_uchar;

#[repr(C)]
pub struct _xmlDoc {
    _private: *mut c_void,
}

pub type xmlDocPtr = *mut _xmlDoc;

#[repr(C)]
pub struct _xmlNode {
    pub _private: *mut c_void,
    pub type_: c_int,
    pub name: *const xmlChar,
    pub children: *mut _xmlNode,
    pub last: *mut _xmlNode,
    pub parent: *mut _xmlNode,
    pub next: *mut _xmlNode,
    pub prev: *mut _xmlNode,
    pub doc: *mut _xmlDoc,
    pub ns: *mut c_void,
    pub content: *mut xmlChar,
    pub properties: *mut c_void,
    pub nsDef: *mut c_void,
    pub psvi: *mut c_void,
    pub line: u16,
    pub extra: u16,
}

pub type xmlNodePtr = *mut _xmlNode;

pub const XML_ELEMENT_NODE: c_int = 1;
pub const XML_COMMENT_NODE: c_int = 8;

pub const HTML_PARSE_RECOVER: c_int = 1 << 0;
pub const HTML_PARSE_NOERROR: c_int = 1 << 5;
pub const HTML_PARSE_NOWARN: c_int = 1 << 6;
pub const HTML_PARSE_NONET: c_int = 1 << 11;

unsafe extern "C" {
    pub fn htmlReadMemory(
        buffer: *const c_char,
        size: c_int,
        URL: *const c_char,
        encoding: *const c_char,
        options: c_int,
    ) -> xmlDocPtr;

    pub fn xmlDocGetRootElement(doc: xmlDocPtr) -> xmlNodePtr;
    pub fn xmlFreeDoc(doc: xmlDocPtr);
    pub fn xmlNodeGetContent(cur: xmlNodePtr) -> *mut xmlChar;
    pub fn xmlUnlinkNode(cur: xmlNodePtr);
    pub fn xmlFreeNode(cur: xmlNodePtr);
    pub static mut xmlFree: Option<unsafe extern "C" fn(*mut c_void)>;
}

/// Free a pointer allocated by libxml2.
pub fn xml_free(ptr: *mut c_void) {
    unsafe {
        if let Some(free_fn) = xmlFree {
            free_fn(ptr);
        } else {
            free(ptr);
        }
    }
}

/// A parsed HTML document.
pub struct Document {
    ptr: xmlDocPtr,
}

impl Document {
    /// Parse an HTML document from bytes using libxml2.
    pub fn parse_html(input: &[u8], options: c_int) -> Option<Self> {
        let doc = unsafe {
            htmlReadMemory(
                input.as_ptr() as *const c_char,
                input.len() as c_int,
                std::ptr::null(),
                std::ptr::null(),
                options,
            )
        };
        if doc.is_null() {
            None
        } else {
            Some(Self { ptr: doc })
        }
    }

    /// Get the root element of the document.
    pub fn root(&self) -> Option<Node<'_>> {
        let node = unsafe { xmlDocGetRootElement(self.ptr) };
        if node.is_null() {
            None
        } else {
            Some(Node {
                ptr: node,
                _doc: PhantomData,
            })
        }
    }
}

impl Drop for Document {
    fn drop(&mut self) {
        unsafe { xmlFreeDoc(self.ptr) }
    }
}

/// A node in the DOM tree.
pub struct Node<'a> {
    ptr: xmlNodePtr,
    _doc: PhantomData<&'a Document>,
}

impl<'a> Node<'a> {
    /// Type of the node as defined by libxml2 constants.
    pub fn node_type(&self) -> c_int {
        unsafe { (*self.ptr).type_ }
    }

    /// Name of the node as a lowercase ASCII string.
    pub fn name(&self) -> Option<String> {
        let name_ptr = unsafe { (*self.ptr).name } as *const c_char;
        if name_ptr.is_null() {
            None
        } else {
            Some(unsafe { CStr::from_ptr(name_ptr) }.to_string_lossy().to_ascii_lowercase())
        }
    }

    /// First child of the node.
    pub fn children(&self) -> Option<Node<'a>> {
        let child = unsafe { (*self.ptr).children };
        if child.is_null() {
            None
        } else {
            Some(Node {
                ptr: child,
                _doc: PhantomData,
            })
        }
    }

    /// Next sibling of the node.
    pub fn next(&self) -> Option<Node<'a>> {
        let next = unsafe { (*self.ptr).next };
        if next.is_null() {
            None
        } else {
            Some(Node {
                ptr: next,
                _doc: PhantomData,
            })
        }
    }

    /// Unlink the node from the document.
    pub fn unlink(&self) {
        unsafe { xmlUnlinkNode(self.ptr) }
    }

    /// Free the node memory.
    pub fn free(self) {
        unsafe { xmlFreeNode(self.ptr) }
    }

    /// Get the textual content of the node.
    pub fn content(&self) -> Option<Vec<u8>> {
        let ptr = unsafe { xmlNodeGetContent(self.ptr) };
        if ptr.is_null() {
            return None;
        }
        let bytes = unsafe { CStr::from_ptr(ptr as *const c_char).to_bytes().to_vec() };
        xml_free(ptr as *mut c_void);
        Some(bytes)
    }
}
