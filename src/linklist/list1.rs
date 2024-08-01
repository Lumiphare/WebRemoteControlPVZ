use std::{cell::RefCell, error::Error, rc::Rc};
enum Node {
    ListNode(i32, Rc<RefCell<Node>>),
    Nil
}
struct List {
    head: Rc<RefCell<Node>>,
    tail: Rc<RefCell<Node>>,
    len:u8 // head表示dummy head
}
impl List {
    pub fn new() -> List {
        let head: Rc<RefCell<Node>> = Rc::new(RefCell::new(Node::ListNode(0, Rc::new(RefCell::new(Node::Nil)))));
        List {
            tail:Rc::clone(&head),
            head,
            len:0
        }
    }
    /// 增加链表节点
    pub fn add(&mut self, val:i32) {
        let n_node: Rc<RefCell<Node>> = Rc::new(RefCell::new(Node::ListNode(val, Rc::new(RefCell::new(Node::Nil)))));
        if let Node::ListNode(_, ref mut val) = *(*self.tail).borrow_mut() {
            *val = n_node.clone();
        }
        self.tail = n_node;
        self.len += 1;
    }
    /// 删除头节点
    pub fn remove_first(&mut self) -> Result<(), Box<dyn Error>> {
        if self.get_len() == 0 {
            return Err("链表为空，删除头节点失败".into());  // 将&str类型转换为Box<dyn Error>
        }
        let c;
        if let Node::ListNode(_, ref next) = *(*self.head).borrow_mut() {
            c = next.clone();
        } else {
            return Err("错误".into());
        }
        self.head = c;
        self.len -= 1;
        Ok(())
    }
    /// 转换为数组
    pub fn to_vec(&self) -> Vec<i32> {
        let mut res: Vec<i32> = Vec::new();
        if self.len == 0 {
            return res;
        }
        let mut p = self.head.clone();
        let mut is_dump_head: bool = true;
        loop {
            let c;
            if let Node::ListNode(ref val, ref next) = *p.borrow() {
                if is_dump_head {   // dummy head的值不计入
                    is_dump_head = false;
                } else {
                    res.push(*val);
                }
                c = next.clone();
            } else {
                break;
            }
            p = c;
        }
        res
    }
    /// 获取链表长度
    pub fn get_len(&self) -> u8{
        self.len
    }
}

#[cfg(test)]
mod test {
    use crate::linklist::list1::{List};
    #[test]
    fn test1() {  // 测试
        let mut list = List::new();
        list.add(1);
        list.add(2);
        println!("{}", list.get_len());
        println!("{:?}", list.to_vec());
        list.remove_first().unwrap();
        println!("{:?}", list.to_vec());
    }
}