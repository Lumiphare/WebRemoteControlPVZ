const appcontrol = (() => {
    const fullscreen = document.getElementById("fullscreen");
    const style = document.createElement('style');
    style.textContent = `
        body, html {
            margin: 0;
            padding: 0;
            overflow: hidden; /* 禁止滚动 */
            height: 100%;
            width: 100%;
            display: flex;
            justify-content: center;
            align-items: center;
            /* background-color: white; */
        }
        #app-screen {
            position: absolute;
            transform-origin: center;
            transform: rotate(90deg); /* 初始竖屏旋转90度 */
            width: 100vh; /* 根据竖屏时的高度设置宽度 */
            height: 100vw; /* 根据竖屏时的宽度设置高度 */
            object-fit: contain; /* 确保视频内容覆盖整个区域 */
        }
  ` ;
    fullscreen.addEventListener("click", () => {
        document.head.appendChild(style);
    });

})();