//pub mod singly_linked_list;

//pub use singly_linked_list::SLLBucket;
pub type SLLBucket<K,V,A> = linked_lists::singly_linked_list::SinglyLinkedList<(K,V), A>;