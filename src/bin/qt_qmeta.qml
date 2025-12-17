import QtQuick 2.6
import QtQuick.Window 2.2
import QtQuick.Controls 2.0
import QtQuick.Layouts 1.3
import FileExplorer 1.0
import Qt.labs.folderlistmodel 2.15

Window {
    id: root
    visible: true
    width: 900
    height: 700
    title: "Android File Explorer (ADB)"

    // explorer is provided by Rust as a context property
    
    // Model to store file listing data
    ListModel {
        id: fileListModel
    }

    // Monitor path changes and load files
    Connections {
        target: explorer
        function onStatusChanged() {
            console.log("Status updated:", explorer.status_message)
        }
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 10
        spacing: 10

        // Toolbar
        Rectangle {
            Layout.fillWidth: true
            height: 50
            color: "#f5f5f5"
            border.color: "#cccccc"
            border.width: 1

            RowLayout {
                anchors.fill: parent
                anchors.margins: 5
                spacing: 10

                Button {
                    text: "⬅ Back"
                    width: 80
                    onClicked: {
                        var currentPath = pathField.text
                        var lastSlash = currentPath.lastIndexOf('/')
                        if (lastSlash > 0) {
                            var parentPath = currentPath.substring(0, lastSlash)
                            if (parentPath === "") parentPath = "/"
                            pathField.text = parentPath
                            loadFiles(parentPath)
                        }
                    }
                }

                TextField {
                    id: pathField
                    Layout.fillWidth: true
                    text: "/"
                    placeholderText: "Enter file path..."
                    onAccepted: loadFiles(text)
                }

                Button {
                    text: "Refresh"
                    width: 80
                    onClicked: {
                        loadFiles(pathField.text)
                    }
                }

                Button {
                    text: "Home"
                    width: 60
                    onClicked: {
                        pathField.text = "/"
                        loadFiles("/")
                    }
                }
            }
        }

        // Quick navigation buttons
        RowLayout {
            Layout.fillWidth: true
            height: 40
            spacing: 10

            Button {
                text: "/data"
                onClicked: {
                    pathField.text = "/data"
                    loadFiles("/data")
                }
            }

            Button {
                text: "/sdcard"
                onClicked: {
                    pathField.text = "/sdcard"
                    loadFiles("/sdcard")
                }
            }

            Button {
                text: "/system"
                onClicked: {
                    pathField.text = "/system"
                    loadFiles("/system")
                }
            }

            Button {
                text: "/cache"
                onClicked: {
                    pathField.text = "/cache"
                    loadFiles("/cache")
                }
            }

            Item { Layout.fillWidth: true }
        }

        // Status bar
        Rectangle {
            Layout.fillWidth: true
            height: 25
            color: "#e8e8e8"
            border.color: "#cccccc"
            border.width: 1

            Text {
                anchors.fill: parent
                anchors.margins: 5
                text: explorer.status_message
                font.pixelSize: 11
                verticalAlignment: Text.AlignVCenter
            }
        }

        // File list display
        Rectangle {
            Layout.fillWidth: true
            Layout.fillHeight: true
            border.color: "#cccccc"
            border.width: 1
            color: "white"

            ListView {
                id: fileListView
                anchors.fill: parent
                clip: true
                model: fileListModel

                delegate: ItemDelegate {
                    width: fileListView.width
                    height: 35

                    RowLayout {
                        anchors.fill: parent
                        anchors.margins: 5
                        spacing: 15

                        Text {
                            text: model.isDir ? "📁" : "📄"
                            font.pixelSize: 18
                        }

                        Text {
                            Layout.fillWidth: true
                            text: model.name
                            font.family: "Courier"
                            font.pixelSize: 11
                        }

                        Text {
                            text: model.size
                            font.pixelSize: 10
                            color: "gray"
                            width: 80
                            horizontalAlignment: Text.AlignRight
                        }
                    }

                    onClicked: {
                        if (model.isDir) {
                            // Navigate into directory
                            var newPath = explorer.current_path
                            if (newPath === "/") {
                                newPath = "/" + model.name
                            } else {
                                newPath = newPath + "/" + model.name
                            }
                            loadFiles(newPath)
                        }
                    }

                    onDoubleClicked: {
                        if (!model.isDir) {
                            console.log("File selected:", model.name)
                        }
                    }
                }

                ScrollBar.vertical: ScrollBar {
                    active: true
                }
            }
        }

        // Footer
        RowLayout {
            Layout.fillWidth: true
            height: 40
            spacing: 10

            Text {
                text: "Path: " + pathField.text
                font.pixelSize: 11
            }

            Item { Layout.fillWidth: true }

            Button {
                text: "Copy Path"
                onClicked: {
                    // Copy to clipboard (platform dependent)
                }
            }

            Button {
                text: "Exit"
                onClicked: Qt.quit()
            }
        }
    }

    // Parse file listing from Rust backend format
    function parseFileList(fileListString) {
        fileListModel.clear()
        var lines = fileListString.split('\n')
        var count = 0
        
        for (var i = 0; i < lines.length; i++) {
            var line = lines[i].trim()
            if (line.length === 0) continue
            
            // Parse format: "[D] name                    size"
            var match = line.match(/^\[(.)\]\s+(\S+)\s+(\S+)$/)
            if (match) {
                var type = match[1]
                var name = match[2]
                var size = match[3]
                
                fileListModel.append({
                    "name": name,
                    "size": size,
                    "isDir": type === "D",
                    "fullPath": explorer.current_path + (explorer.current_path === "/" ? "" : "/") + name
                })
                count++
            }
        }
        
        return count
    }
    
    // JavaScript helper function - load files from ADB
    function loadFilesWithData(path) {
        console.log("Loading files from path:", path)
        explorer.current_path = path
        explorer.status_message = "Loading " + path + "..."
        
        // Get files from Rust backend
        var filesStr = explorer.file_list
        
        // Parse and populate the model
        var count = parseFileList(filesStr)
        explorer.status_message = "Loaded " + count + " items from " + path
    }
    
    function loadFiles(path) {
        loadFilesWithData(path)
    }

    Component.onCompleted: {
        console.log("UI initialized with explorer object:", explorer)
        console.log("Initial path:", explorer.current_path)
        console.log("Initial file_list length:", explorer.file_list.length)
        
        // Parse the file list that was pre-loaded by Rust
        var count = parseFileList(explorer.file_list)
        console.log("Parsed", count, "files from root directory")
        
        // Ready for user interaction
        Qt.callLater(function() {
            console.log("File explorer ready")
        })
    }
}
