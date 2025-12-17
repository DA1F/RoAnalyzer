import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Qt.labs.folderlistmodel

ApplicationWindow {
    visible: true
    width: 1200
    height: 800
    title: qsTr("Файловый менеджер")

    header: ToolBar {
        Row {
            id: rowToolBar
            anchors.fill: parent

            ToolButton {
                id: upButton
                icon.source: "qrc:/res/up_folder.png"
                onClicked: fileSystemModel.up()
            }

            ToolButton {
                id: iconsButton
                text: qsTr("Иконки")
                checkable: true
                checked: true
                onClicked: stackView.currentIndex = 0
                ButtonGroup.group: layoutButtonGroup
            }

            ToolButton {
                id: listButton
                text: qsTr("Список")
                checkable: true
                onClicked: stackView.currentIndex = 1
                ButtonGroup.group: layoutButtonGroup
            }

            ToolButton {
                id: tableButton
                text: qsTr("Таблица")
                checkable: true
                onClicked: stackView.currentIndex = 2
                ButtonGroup.group: layoutButtonGroup
            }
        }
    }

    property string pathSelectedFile: ""

    ButtonGroup { id: layoutButtonGroup }


    Menu {
        id: contextMenu

        MenuItem {
            text: qsTr("Открыть")
            onTriggered: {
                if (fileManager.isFile(pathSelectedFile))
                    Qt.openUrlExternally("file://" + pathSelectedFile)
                else
                    fileSystemModel.cd(pathSelectedFile)
            }

        }

        MenuSeparator {}

        MenuItem {
            text: qsTr("Копировать")
            onTriggered: {
                fileManager.copyFile(pathSelectedFile)
            }
        }
        MenuItem {
            text: qsTr("Вырезать")
            onTriggered: {
                fileManager.cutFile(pathSelectedFile)
            }
        }
        MenuItem {
            text: qsTr("Вставить")
            onTriggered: {
                fileManager.pasteFile(fileSystemModel.currentPath)
                fileSystemModel.refresh()
            }
        }
        MenuItem {
            text: qsTr("Удалить")
            onTriggered: {
                fileManager.deleteFile(pathSelectedFile)
                fileSystemModel.refresh()
            }
        }
    }

    Menu {
        id: contextMenu1
        MenuItem {
            text: qsTr("Вставить")
            onTriggered: {
                fileManager.pasteFile(fileSystemModel.currentPath)
                fileSystemModel.refresh()
            }
        }
    }

TabBar {
            id: bar
            width: parent.width
            height: 40
            TabButton {
                text: qsTr("Home")
            }
            TabButton {
                text: qsTr("Discover")
            }
            TabButton {
                text: qsTr("Activity")
            }
        }

        StackLayout {
            width: parent.width
            currentIndex: bar.currentIndex
            Item {
                id: homeTab
            }
            Item {
                id: discoverTab
            }
            Item {
                id: activityTab
            }
        }


    Rectangle {
        color: "white"
        anchors.fill: parent
        anchors.margins: 2 * 4 + 4

        StackLayout {
            id: stackView
            anchors.fill: parent

            Item {
                MouseArea {
                    anchors.fill: parent
                    acceptedButtons: Qt.LeftButton | Qt.RightButton
                    onClicked: {
                        listView.currentIndex = -1
                        pathSelectedFile = ""
                        if (mouse.button == Qt.RightButton)
                            contextMenu1.popup()
                    }
                }

                FmGridView {}
            }

            Item {
                Layout.fillWidth: true
                Layout.fillHeight: true

                MouseArea {
                    anchors.fill: parent
                    acceptedButtons: Qt.LeftButton | Qt.RightButton
                    onClicked: {
                        listView.currentIndex = -1
                        pathSelectedFile = ""
                        if (mouse.button == Qt.RightButton)
                            contextMenu1.popup()
                    }
                }

                FmListView {}
            }

            FmTableView {
                Layout.fillWidth: true
                Layout.fillHeight: true
            }
        }
    }
}