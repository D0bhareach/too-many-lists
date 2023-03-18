use std::rc::Rc;

// in this module an immutable thread not save stack will be implimented
pub struct List<T> {
    head: Link<T>,
}

type Link<T> = Option<Rc<Node<T>>>;

struct Node<T> {
    elem: T,
    next: Link<T>,
}

impl<T> List<T> {
    pub fn new() -> Self {
        List { head: None }
    }

    pub fn prepend(&self, elem: T) -> List<T> {
        List {
            head: Some(Rc::new(Node {
                elem: elem,
                /* Lucky we our head = Option<Rc<Node<T>>>, this way we only incrementing
                 * counter on Rc. Relatively cheap operation. On each prepend previous
                 * List fat pointer Rc is returned.
                 * fn clone(&self) -> Self {
                        match self {
                            Some(x) => Some(x.clone()),
                            None => None,
                        }
                    }
                */
                next: self.head.clone(),
            })),
        }
    }

    pub fn tail(&self) -> List<T> {
        List {
            // List { head: self.head.as_ref().map(|node| node.next.clone()) }
            // in first implementation is not working because we need to get
            // Option out of Option, as_ref Converts from &Option<T> to Option<&T>.
            // so we get Option and then..
            head: self.head.as_ref().and_then(|node| node.next.clone()),
            // get pointer to current element, get Node, get rest of List, assign as current
        }
    }

    pub fn head(&self) -> Option<&T> {
        self.head.as_ref().map(|node| &node.elem)
    }
}

pub struct Iter<'a, T> {
    next: Option<&'a Node<T>>,
}

impl<T> List<T> {
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            next: self.head.as_deref(),
        }
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|node| {
            self.next = node.next.as_deref();
            &node.elem
        })
    }
}

impl<T> Drop for List<T> {
    fn drop(&mut self) {
        // get Option of Rc
        let mut head = self.head.take();
        while let Some(node) = head {
            // node must either have exactly one strong pointer to other node and so on
            // or it's the last one. If node has pointer it taken not borrowed and when
            // while loop is reenters this taken value is collected by operating system
            // and memory is freed (exact behaviour of OS is not clear to me yet)
            if let Ok(mut node) = Rc::try_unwrap(node) {
                head = node.next.take();
            } else {
                break;
            }
        }
    }
}
