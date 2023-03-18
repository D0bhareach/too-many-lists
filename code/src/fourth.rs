#![allow(dead_code)]
/// Bad safe Deque
use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;

pub struct List<T> {
    head: Link<T>,
    tail: Link<T>,
}

type Link<T> = Option<Rc<RefCell<Node<T>>>>;

struct Node<T> {
    elem: T,
    next: Link<T>,
    prev: Link<T>,
}

impl<T> Node<T> {
    fn new(elem: T) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Node {
            elem: elem,
            prev: None,
            next: None,
        }))
    }
}

impl<T> List<T> {
    pub fn new() -> Self {
        List {
            head: None,
            tail: None,
        }
    }

    pub fn push_front(&mut self, elem: T) {
        let new_head = Node::new(elem);
        match self.head.take() {
            Some(old_head) => {
                // new_nead is Rc, Rc.clone makes copy of pointer to value and increase counter.
                old_head.borrow_mut().prev = Some(new_head.clone());
                new_head.borrow_mut().next = Some(old_head);
                self.head = Some(new_head);
            }
            // for the first time when there isn't any element yet.
            None => {
                self.tail = Some(new_head.clone());
                self.head = Some(new_head);
            }
        }
    }

    pub fn pop_front(&mut self) -> Option<T> {
        // head - Option, old_head - RefCell because Rc<T> automatically dereferences to T
        self.head.take().map(|old_head| {
            match old_head.borrow_mut().next.take() {
                // new_head is RefCell
                Some(new_head) => {
                    // take old_head never reassign it means remove prev.
                    // prev = None now.
                    new_head.borrow_mut().prev.take();
                    // List head = new_head, but what is the tail?
                    self.head = Some(new_head);
                }
                None => {
                    self.tail.take();
                }
            }
            // get inner val, to be able to unwrap convert to Option into_inner return Node.
            Rc::try_unwrap(old_head).ok().unwrap().into_inner().elem
        })
    }

    // http://localhost:3000/fourth-peek.html
    // it's really a lot in it and explanation is lengthy, bottom line RefCell is not easy
    // and must learn api.
    pub fn peek_front(&self) -> Option<Ref<T>> {
        self.head
            .as_ref()
            // Converts from &Option<T> to Option<&T> in this context as if
            // RefCell.borrow()
            .map(|node| Ref::map(node.borrow(), |node| &node.elem))
        //&RefCell, so now when we have different pointer to RefCell we borrow out Node.
    }

    // Methods below are almost copy paste of method for the front just have to reverse
    // nodes.
    pub fn push_back(&mut self, elem: T) {
        let new_tail = Node::new(elem);
        match self.tail.take() {
            Some(old_tail) => {
                old_tail.borrow_mut().next = Some(new_tail.clone());
                new_tail.borrow_mut().prev = Some(old_tail);
                self.tail = Some(new_tail);
            }
            None => {
                self.head = Some(new_tail.clone());
                self.tail = Some(new_tail);
            }
        }
    }

    pub fn pop_back(&mut self) -> Option<T> {
        self.tail.take().map(|old_tail| {
            match old_tail.borrow_mut().prev.take() {
                Some(new_tail) => {
                    new_tail.borrow_mut().next.take();
                    self.tail = Some(new_tail);
                }
                None => {
                    self.head.take();
                }
            }
            Rc::try_unwrap(old_tail).ok().unwrap().into_inner().elem
        })
    }

    pub fn peek_back(&self) -> Option<Ref<T>> {
        self.tail
            .as_ref()
            .map(|node| Ref::map(node.borrow(), |node| &node.elem))
    }

    // Methods for mutable operations
    pub fn peek_back_mut(&mut self) -> Option<RefMut<T>> {
        self.tail
            .as_ref()
            .map(|node| RefMut::map(node.borrow_mut(), |node| &mut node.elem))
    }

    pub fn peek_front_mut(&mut self) -> Option<RefMut<T>> {
        self.head
            .as_ref()
            .map(|node| RefMut::map(node.borrow_mut(), |node| &mut node.elem))
    }
}

impl<T> Drop for List<T> {
    fn drop(&mut self) {
        while self.pop_front().is_some() {}
    }
}

// ITERATION
pub struct IntoIter<T>(List<T>);

impl<T> List<T> {
    pub fn into_iter(self) -> IntoIter<T> {
        IntoIter(self)
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop_front()
    }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<T> {
        self.0.pop_back()
    }
}
// Iter and IterMut are not possible on current set up with Cell and Rc simply
// because of how they are implimented in Rust. Read chapter on fourth-iteration or Bad Deque.
