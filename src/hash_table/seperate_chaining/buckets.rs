//pub mod singly_linked_list;

//pub use singly_linked_list::SLLBucket;
pub type SLLBucket<K, V, A> = crate::linked_lists::SinglyLinkedList<(K, V), A>;
