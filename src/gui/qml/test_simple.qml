import QtQuick 2.6
import QtQuick.Window 2.2
import QtQuick.Controls 2.0
import QtQuick.Layouts 1.3
import DemoFileExplorer 1.0

Window {
    id: root
    visible: true
    width: 600
    height: 400
    title: "File Icon Test - Native-like Icons"

    DemoFileExplorer { id: explorer }

    // Sample file data
    ListModel {
        id: sampleFiles
        ListElement { name: "document.pdf"; isDir: false }
        ListElement { name: "photo.jpg"; isDir: false }
        ListElement { name: "music.mp3"; isDir: false }
        ListElement { name: "video.mp4"; isDir: false }
        ListElement { name: "archive.zip"; isDir: false }
        ListElement { name: "code.rs"; isDir: false }
        ListElement { name: "app.apk"; isDir: false }
        ListElement { name: "Documents"; isDir: true }
        ListElement { name: "Pictures"; isDir: true }
        ListElement { name: "readme.txt"; isDir: false }
        ListElement { name: "presentation.pptx"; isDir: false }
        ListElement { name: "spreadsheet.xlsx"; isDir: false }
        ListElement { name: "source.py"; isDir: false }
        ListElement { name: "index.html"; isDir: false }
        ListElement { name: "library.so"; isDir: false }
    }

    // JavaScript function to get file icon
    function getFileIcon(filename) {
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

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 20

        Text {
            text: "📁 File Icon Examples (Native-like Emojis)"
            font.pixelSize: 20
            font.bold: true
            Layout.alignment: Qt.AlignHCenter
        }

        Rectangle {
            Layout.fillWidth: true
            height: 2
            color: "#e0e0e0"
        }

        ScrollView {
            Layout.fillWidth: true
            Layout.fillHeight: true
            
            ListView {
                id: fileListView
                spacing: 2
                clip: true
                
                model: sampleFiles

                delegate: ItemDelegate {
                    width: fileListView.width - 20
                    height: 50

                    background: Rectangle {
                        color: {
                            if (fileListView.currentIndex === index) return "#e3f2fd"
                            else if (hovered) return "#f5f5f5"
                            else return "white"
                        }
                        border.color: "#e0e0e0"
                        border.width: 1
                        radius: 4
                    }

                    RowLayout {
                        anchors.fill: parent
                        anchors.margins: 8
                        spacing: 12

                        // Icon
                        Text {
                            text: model.isDir ? "📁" : getFileIcon(model.name)
                            font.pixelSize: 28
                            Layout.preferredWidth: 35
                            Layout.alignment: Qt.AlignVCenter
                        }

                        // File name
                        Text {
                            Layout.fillWidth: true
                            text: model.name
                            font.pixelSize: 14
                            font.bold: model.isDir
                            color: model.isDir ? "#1976d2" : "#212121"
                            elide: Text.ElideRight
                            Layout.alignment: Qt.AlignVCenter
                        }

                        // Type label
                        Rectangle {
                            Layout.preferredWidth: 60
                            Layout.preferredHeight: 22
                            color: model.isDir ? "#e3f2fd" : "#f5f5f5"
                            radius: 11
                            
                            Text {
                                anchors.centerIn: parent
                                text: model.isDir ? "Folder" : "File"
                                font.pixelSize: 10
                                color: model.isDir ? "#1976d2" : "#757575"
                            }
                        }
                    }

                    onClicked: {
                        fileListView.currentIndex = index
                    }
                }
            }
        }

        Rectangle {
            Layout.fillWidth: true
            height: 1
            color: "#e0e0e0"
        }

        RowLayout {
            Layout.fillWidth: true
            spacing: 10

            Text {
                text: "💡"
                font.pixelSize: 16
            }

            Text {
                Layout.fillWidth: true
                text: "Icons use emoji characters that look similar to native OS icons"
                font.pixelSize: 11
                color: "#757575"
                wrapMode: Text.WordWrap
            }
        }
    }
}
