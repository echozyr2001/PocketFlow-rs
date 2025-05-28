//! 组合式架构模块
//! 
//! 将 PocketFlow-rs 从继承式设计转换为组合式设计。
//! 每个功能都分解为独立、可组合的组件。

pub mod behaviors;
pub mod node;
pub mod builder;

pub use behaviors::*;
pub use node::*;
pub use builder::*;