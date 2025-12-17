import QtQuick

ListView {
    id: listView
    anchors.fill: parent

    model: ListModel {
        ListElement {
            name: "Users"
            type:"Folder"
            size:"4.3 MB"
            modifed:"2024-06-01"
            path:"/srs/"
        }
        ListElement {
            name: "Users2"
            type:"Folder"
            size:"4.3 MB"
            modifed:"2024-06-01"
            path:"/srs/"
        }
        
    }
    delegate: FmListViewDelegate {}
    highlight: Rectangle { color: "lightsteelblue"; radius: 5 }

    flickableChildren: MouseArea {
        anchors.fill: parent
        acceptedButtons: Qt.LeftButton | Qt.RightButton
        onClicked: {
            listView.currentIndex = -1
            pathSelectedFile = ""
            if (mouse.button == Qt.RightButton)
                contextMenu1.popup()
        }
    }

    Component.onCompleted: currentIndex = -1
}
