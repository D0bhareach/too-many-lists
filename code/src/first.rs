#![allow(dead_code)]
use std::mem;

#[derive(Debug)]
pub struct List {
    head: Link,
}

#[derive(Debug)]
enum Link {
    Empty,
    More(Box<Node>),
}

#[derive(Debug)]
struct Node {
    elem: i32,
    next: Link,
}

impl List {
    pub fn new() -> Self {
        List { head: Link::Empty }
    }

    // The only thing that interesting here is how we assign next value to a Node when it's
    // created. We need next would hold all previous List, but if we try to assign it like:
    // `next: self.head to get a pointer to first item of linked list we will get error message:
    // "cannot move out of borrowed content". I need to replace a value in borrowed reference to
    // List with new value, and not only this, to assign value to next it must be value must be
    // owned. As happens to be mem::replace does exactly that. It replaces value of mutable borrow
    // to new value and return previous value as owned.
    pub fn push(&mut self, elem: i32) {
        let node = Node {
            elem: elem,
            // replace moves Link in head returns previos value.
            next: mem::replace(&mut self.head, Link::Empty),
        };

        let link: Link = Link::More(Box::new(node));
        self.head = link;
    }

    pub fn pop(&mut self) -> Option<i32> {
        match mem::replace(&mut self.head, Link::Empty) {
            Link::Empty => None,
            Link::More(node) => {
                self.head = node.next;
                Some(node.elem)
            }
        }
    }
}

// Implementing cleanup must free memory used by List
impl Drop for List {
    // I will not write long explanation here. Just look in: https://rust-unofficial.github.io/too-many-lists/first-drop.html
    // Another thing apparently Enum empty value (without anything to drop) will be dropped auto
    // matically when List goes out of scope. So we go by all nodes of list and reassign them to
    // an empty value. Box with value is going out of scope and will be dropped.
    // more about Box drop: https://doc.rust-lang.org/stable/std/ops/trait.Drop.html#tymethod.drop
    fn drop(&mut self) {
        let mut cur_link = mem::replace(&mut self.head, Link::Empty);
        // `while let` == "do this thing until this pattern doesn't match"
        while let Link::More(mut boxed_node) = cur_link {
            cur_link = mem::replace(&mut boxed_node.next, Link::Empty);
            // boxed_node goes out of scope and gets dropped here;
            // but its Node's `next` field has been set to Link::Empty
            // so no unbounded recursion occurs.
        }
    }
}
