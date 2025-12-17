import QtQuick 2.15



Item {
    id: fileItem
    width: grid.cellWidth - 8
    height: grid.cellHeight - 8

    function getFileIcon(filename) {
        // TODO provide it from Rust side 
        var name = filename.toLowerCase();
        var ext = "";
        var dotIndex = name.lastIndexOf('.');
        if (dotIndex >= 0) {
            ext = name.substring(dotIndex + 1);
        }
        
        // Map extensions to emoji icons (closest to native look)
        var iconMap = {
            // Images
            "jpg": "🖼️", "jpeg": "🖼️", "png": "🖼️", "gif": "🖼️", "bmp": "🖼️", "svg": "🖼️", "webp": "🖼️",
            // Videos
            "mp4": "🎬", "avi": "🎬", "mkv": "🎬", "mov": "🎬", "wmv": "🎬", "flv": "🎬", "webm": "🎬",
            // Audio
            "mp3": "🎵", "wav": "🎵", "flac": "🎵", "aac": "🎵", "ogg": "🎵", "m4a": "🎵",
            // Documents
            "pdf": "📕",
            "doc": "📘", "docx": "📘",
            "xls": "📗", "xlsx": "📗",
            "ppt": "📙", "pptx": "📙",
            "txt": "📄", "md": "📄",
            // Archives
            "zip": "📦", "rar": "📦", "7z": "📦", "tar": "📦", "gz": "📦",
            // Code
            "rs": "📝", "py": "📝", "js": "📝", "ts": "📝", "java": "📝", "cpp": "📝", "c": "📝", "h": "📝",
            "html": "📋", "css": "📋", "xml": "📋", "json": "📋", "yaml": "📋", "yml": "📋",
            // Android
            "apk": "📱",
            "dex": "⚙️",
            "so": "🔧"
        };
        
        return iconMap[ext] || "📄";
    }

    Column {
        anchors.fill: parent

        Text {
            text: getFileIcon(model.name)
            font.pixelSize: 60
            anchors.horizontalCenter: parent.horizontalCenter
        }

        Text {
            width: parent.width
            anchors.left: parent.left
            anchors.right: parent.right
            text: name
            horizontalAlignment: Text.AlignHCenter
            elide: Text.ElideRight
            wrapMode: (grid.currentIndex == index) ? Text.WrapAnywhere : Text.NoWrap
        }
    }

    MouseArea {
        anchors.fill: parent
        acceptedButtons: Qt.LeftButton | Qt.RightButton
        onClicked: {
            grid.currentIndex = index
            pathSelectedFile = path
            if (mouse.button == Qt.RightButton)
                contextMenu.popup()
        }
        onDoubleClicked: 
        {
            //TODO: download the file from adb to a working directory 
            if (fileManager.isFile(path))
                Qt.openUrlExternally("file://" + path)
            else fileSystemModel.cd(path)
        }
    }
}

