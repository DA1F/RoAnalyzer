import QtQuick
import QtQuick.Controls
import QtQuick.Controls.Universal

Control {
    id: control
    
    property alias currentIndex: tabRow.currentIndex
    property alias count: repeater.count
    
    default property alias tabs: repeater.model
    
    background: Rectangle {
        color: "#F5F5F5"
        border.width: 0
        
        Rectangle {
            anchors.bottom: parent.bottom
            anchors.left: parent.left
            anchors.right: parent.right
            height: 1
            color: "#E0E0E0"
        }
    }
    
    contentItem: Row {
        id: tabRow
        property int currentIndex: 0
        spacing: 0
        
        Repeater {
            id: repeater
            
            TabButton {
                id: tabButton
                text: modelData
                width: implicitWidth
                padding: 12
                
                background: Rectangle {
                    color: tabButton.checked ? "#FFFFFF" : "transparent"
                    radius: 8
                    
                    Rectangle {
                        anchors.bottom: parent.bottom
                        anchors.left: parent.left
                        anchors.right: parent.right
                        height: 3
                        color: tabButton.checked ? "#2196F3" : "transparent"
                        radius: 1.5
                    }
                    
                    Rectangle {
                        anchors.fill: parent
                        color: !tabButton.checked ? "#FAFAFA" : "transparent"
                        radius: 8
                        opacity: 0
                        
                        Behavior on opacity {
                            NumberAnimation { duration: 150 }
                        }
                    }
                }
                
                contentItem: Text {
                    text: tabButton.text
                    font.pixelSize: 13
                    font.weight: tabButton.checked ? Font.Medium : Font.Normal
                    color: tabButton.checked ? "#2196F3" : "#666666"
                    horizontalAlignment: Text.AlignHCenter
                    verticalAlignment: Text.AlignVCenter
                }
                
                onClicked: tabRow.currentIndex = index
            }
        }
    }
}
