// Component BackendObject {}

import QtQuick // Or whatever QtQuick version your Qt 6.10.1 supports
import QtQuick.Controls // THIS IS CRUCIAL for TreeView in Qt 6
import QtQml.Models
import QtQuick.Controls.Basic
import QtQuick.Layouts
import Qt.labs.qmlmodels
import AndroidFileExplorer 1.0


ColumnLayout {
    id :roFSView
    anchors.fill: parent
    spacing: 0
    property bool useGridView: true

    AndroidFileExplorer {
        id: explorer
        current_path: "/data/data"
        Component.onCompleted: {
            explorer.refresh()
            var parsed_data = JSON.parse(explorer.json_data)
            treeModel.rows = parsed_data["rows"]
            fileTreeView.expand(0)
        }
    }


    // Toolbar
    ToolBar {

        Layout.fillWidth: true
        Layout.preferredHeight: 50

        RowLayout {
            anchors.fill: parent
            spacing: 10

            Button {
                id: refreshButton
                Layout.preferredWidth: 40  // Square dimensions
                Layout.preferredHeight: 40
                text: "🔄"  // Icon only, no text
                contentItem: Text {
                    text: refreshButton.text // Link to the button's text property
                    font.pixelSize: 22
                    anchors.fill: parent
                    horizontalAlignment: Text.AlignHCenter
                    verticalAlignment: Text.AlignVCenter
                    anchors.leftMargin: 3

                }
                // Optional: Add tooltip for accessibility
                ToolTip.visible: hovered
                ToolTip.text: "Refresh"
                onClicked: explorer.refresh()
            }

            TextField {
                id: addressbar
                text: explorer.current_path
                Layout.fillWidth: true
                padding: 10
                selectByMouse: true     // Allows mouse selection
                onAccepted: explorer.cd(text)
                background: Rectangle {
                    color: "#f9f9f9"
                    border.color: "#ccc"
                    border.width: 1
                    radius: 5
                }

            }
            Button {
                id: upButton
                Layout.preferredWidth: 40  // Square dimensions
                Layout.preferredHeight: 40
                text: "⬆️"  // Icon only, no text
                font.pixelSize: 40  // Adjust icon size
                // Optional: Add tooltip for accessibility
                ToolTip.visible: hovered
                ToolTip.text: "Go Up"
                contentItem: Text {
                    text: upButton.text // Link to the button's text property
                    font.pixelSize: 22
                    anchors.fill: parent
                    horizontalAlignment: Text.AlignHCenter
                    verticalAlignment: Text.AlignVCenter
                    anchors.leftMargin: 3
                }
                onClicked: explorer.up()
            }

            Button {
                id: toggleViewButton
                Layout.preferredWidth: 40
                Layout.preferredHeight: 40
                text: roFSView.useGridView ? "⬛" : "≣"
                font.pixelSize: 22
                ToolTip.visible: hovered
                ToolTip.text: roFSView.useGridView ? "Switch to list" : "Switch to grid"
                contentItem: Text {
                    text: toggleViewButton.text
                    font.pixelSize: 22
                    anchors.fill: parent
                    horizontalAlignment: Text.AlignHCenter
                    verticalAlignment: Text.AlignVCenter
                }
                onClicked: roFSView.useGridView = !roFSView.useGridView
            }
        }
    }

    // Main content
    SplitView {
        Layout.fillWidth: true
        Layout.fillHeight: true
        orientation: Qt.Horizontal
        font.pixelSize: 14


        // Left panel - File tree
        Rectangle {
            SplitView.preferredWidth: 400
            SplitView.minimumWidth: 150
            ScrollView {
            anchors.fill: parent
                TreeView {
                    id: fileTreeView
                    anchors.fill: parent
                    columnWidthProvider: function(column) {
                        return width; // stretch column to the viewport width
                    }

                    model: TreeModel {
                        id: treeModel
                        TableModelColumn { 
                            display: "name"
                        }
                        rows: []
                    }
                    
                    selectionModel: ItemSelectionModel { 
                        id: itemSelectionModel 
                        onCurrentChanged: {
                            var path = [];
                            var current=currentIndex;
                            while (current.data()) {
                                path.push(current.data());
                                current = current.parent;
                            }
                            explorer.print_lol(path.reverse().join("/"));
                        }
                    }                    
                    delegate: TreeViewDelegate {
                        id: treeDelegate
                        implicitHeight: 22
                        topPadding: 0
                        bottomPadding: 0
                        
                        // macOS-like indentation
                        indentation: 18
                        
                        // Custom indicator (disclosure triangle)
                        indicator: Item {
                            id: indicatorItem
                            implicitWidth: 22
                            implicitHeight: treeDelegate.implicitHeight
                            x: (treeDelegate.depth * treeDelegate.indentation) + 5
                            visible: treeDelegate.hasChildren
                            
                            Text {
                                anchors.centerIn: parent
                                text: treeDelegate.expanded ?  "⌄" : "›"
                                font.pixelSize: 16
                                color: "#666666"
                            }
                        }
                        
                        // Content (icon + text)
                        contentItem: RowLayout {
                            spacing: 4
                            x: (treeDelegate.depth * treeDelegate.indentation) + (treeDelegate.hasChildren ? 20 : 6)
                            Text {
                                text: "📁 "+ treeDelegate.model.display
                                font.pixelSize: 14
                                color: treeDelegate.current ? "#FFFFFF" : "#000000"
                                Layout.fillWidth: true
                            }
                            
                        }
                        
                        // // macOS-like selection background
                        background: Rectangle {
                            width: parent.width
                            color: {
                                if (treeDelegate.current) {
                                    return "#0051D5"
                                }
                                if (treeDelegate.hovered) {
                                    return "#F5F5F5"
                                }
                                return treeDelegate.row % 2 === 0 ? "#EFEFEF" : "white"
                            }
                            
                            Rectangle {
                                anchors.fill: parent
                                color: "transparent"
                                border.color: treeDelegate.current ? "#0051D5" : "transparent"
                                border.width: 0
                            }
                        }
                        
                        // // Change text color when selected
                        // Binding {
                        //     target: ???
                        //     property: "color"
                        //     value: treeDelegate.selected ? "#FFFFFF" : "#000000"
                        // }
                    }
                    
                }
            }


        }
    // Right panel - File details
        Rectangle {
            id: rightPanel
            SplitView.preferredWidth: 600
            SplitView.minimumWidth: 300
            color: "white"
            Loader {
                anchors.fill: parent
                sourceComponent: roFSView.useGridView ? gridComponent : listComponent
            }
            Component { id: gridComponent; FmGridView {} }
            Component { id: listComponent; FmTableView {} }
        }
    }
}