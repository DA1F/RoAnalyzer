import QtQuick 6.10
import QtQuick.Controls 6.10
import QtQuick.Layouts 6.10

ApplicationWindow {
    visible: true
    width: 1200
    height: 800
    title: "Ro Analyser GUI 0.1"

    menuBar: MenuBar {
    id: menuBar
        Menu {
            title: qsTr("&File")
            Action { text: qsTr("&New...") }
            Action { text: qsTr("&Open...") }
            Action { text: qsTr("&Save") }
            Action { text: qsTr("Save &As...") }
            MenuSeparator { }
            Action { text: qsTr("&Quit") }
        }
        Menu {
            title: qsTr("&Edit")
            Action { text: qsTr("Cu&t") }
            Action { text: qsTr("&Copy") }
            Action { text: qsTr("&Paste") }
        }
        Menu {
            title: qsTr("&Help")
            Action { text: qsTr("&About") }
        }
    }
    SplitView {
        anchors.fill: parent
        orientation: Qt.Horizontal
        
        // Left panel - can be resized
        Rectangle {
            color: "lightgray"
            SplitView.minimumWidth: 200
            SplitView.preferredWidth: 600
            SplitView.fillHeight: true
        }
        Rectangle{
            SplitView.minimumWidth: 150
            SplitView.fillHeight: true

            ColumnLayout {
                anchors.fill: parent
                spacing: 0
                
                NativeTabBar {
                    id: bar
                    Layout.fillWidth: true
                    tabs: ["Home", "File System", "Network"]
                    currentIndex: 1
                }

                StackLayout {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    currentIndex: bar.currentIndex
                    
                    Item {
                        id: homeTab
                        Rectangle {
                            color: "white"
                            anchors.fill: parent
                        }
                    }
                    Item {
                        id: fsTab
                        RoFSView {
                            anchors.fill: parent
                        }
                    }
                    Item {
                        id: activityTab
                        Rectangle {
                            color: "green"
                            anchors.fill: parent
                        }
                    }
                }
            }
        }

        
        
    }
}
