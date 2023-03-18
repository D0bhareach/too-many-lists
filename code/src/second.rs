#![allow(dead_code)]
#[derive(Debug)]
pub struct List<T> {
    // List is simple wrapper around type Option.
    head: Link<T>,
}

type Link<T> = Option<Box<Node<T>>>;

#[derive(Debug)]
struct Node<T> {
    elem: T,
    next: Link<T>,
}

impl<T> List<T> {
    pub fn new() -> Self {
        List { head: None }
    }

    // Since we using Option now instead of operating memory on low level we use Option.take()
    // https://doc.rust-lang.org/stable/std/option/enum.Option.html#method.take
    pub fn push(&mut self, elem: T) {
        let node = Node {
            elem: elem,
            next: self.head.take(),
        };

        let link = Some(Box::new(node));
        self.head = link;
    }

    // map on option does something with Option and returns Option
    pub fn pop(&mut self) -> Option<T> {
        self.head.take().map(|node| {
            self.head = node.next;
            node.elem
        })
    }

    pub fn peek(&self) -> Option<&T> {
        self.head.as_ref().map(|node| &node.elem)
    }

    pub fn peek_mut(&mut self) -> Option<&mut T> {
        self.head.as_mut().map(|node| &mut node.elem)
    }
}

// Implementing cleanup must free memory used by List
impl<T> Drop for List<T> {
    // I will not write long explanation here. Just look in: https://rust-unofficial.github.io/too-many-lists/first-drop.html
    // Another thing apparently Enum empty value (without anything to drop) will be
    // dropped automatically when List goes out of scope. So we go by all nodes of list
    // and reassign them to an empty value. Box with value is going out of scope and will
    // be dropped. More about Box drop: https://doc.rust-lang.org/stable/std/ops/trait.Drop.html#tymethod.drop
    fn drop(&mut self) {
        let mut cur_link = self.head.take();
        // `while let` == "do this thing until this pattern doesn't match"
        while let Some(mut boxed_node) = cur_link {
            cur_link = boxed_node.next.take();
            // boxed_node goes out of scope and gets dropped here;
        }
    }
}

// Tuple structs are an alternative form of struct,
// useful for trivial wrappers around other types.
pub struct IntoIter<T>(List<T>);

impl<T> List<T> {
    pub fn into_iter(self) -> IntoIter<T> {
        IntoIter(self)
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        // access fields of a tuple struct numerically
        // every other call to next simply pops head which is Option of boxed value.
        self.0.pop()
    }
}

// implimentation of  Iter for our List.
pub struct Iter<'a, T> {
    next: Option<&'a Node<T>>,
}

impl<T> List<T> {
    pub fn iter<'a>(&'a self) -> Iter<'a, T> {
        Iter {
            // as I understand as_deref creates another reference to original.
            next: self.head.as_deref(),
            // our Iter storing borrowed Node, but head in the List is Option<Box<Node<T>>>
            // need to dereference
        }
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    // first map maps to Iter Option, second map maps to badly named Node Option
    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|node| {
            self.next = node.next.as_deref();
            &node.elem
        })
    }
}
pub struct IterMut<'a, T> {
    next: Option<&'a mut Node<T>>,
}

impl<T> List<T> {
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut {
            next: self.head.as_deref_mut(),
        }
    }
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|node| {
            self.next = node.next.as_deref_mut();
            &mut node.elem
        })
    }
}
