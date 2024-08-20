# Web端远程控制游戏实现

1. **音视频**
    - [x] WebRTC连接成功
    - [x] Web端发送控制视频信号
        - [x] 获取鼠标在视频中的坐标并创建通道发送
    - [ ] 获取声音信号，（FFmpeg 实现）

2. **控制**
    - [x] 使用Rust的windows库的API实现
    - [x] 解决窗口捕获的中文字符问题
        ```rust
        let title_name_u16: Vec<u16> = OsStr::new(title_name)
            .encode_wide()
            .chain(std::iter::once(0)) // 添加 null terminator
            .collect();
        ```
    - [x] 解决有点击反应却不能响应点击问题（点击坐标问题，PostMessageW是窗口内坐标，而且不能直接获得窗口内坐标，必须从屏幕坐标转换ScreenToClient）
    - [x] 显示比例调整，需要根据客户端的分辨率调整控制的位置
        - [x] 获取Video最大坐标和游戏窗口最大坐标

3. **后端**
    - [x] axum返回html界面
    - [x] 局域网成功连接
    - [x] 生成SDP并返回前端
    - [ ] 启动游戏
    - [ ] 使用DXGI完成屏幕录制
    - [ ] 多机运行及负载均衡

4. **前端**
    - [x] 移动端视频显示问题
    - [x] 移动端全屏播放

5. **虚拟化**
    - [ ] XenServer实现虚拟化