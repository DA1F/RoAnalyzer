import QtQuick 6.10
import QtQuick.Controls 6.10
import Qt.labs.qmlmodels 6.10
import QtQuick.Layouts 6.10

Item {
    id: root
    anchors.fill: parent
    property int  rowHeight: 32
    property var headerLabels: ["Name", "Kind", "Size", "Date Modified", "Path"]

    HorizontalHeaderView {
        id: headerView
        syncView: tableView
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.top: parent.top
        delegate: HorizontalHeaderViewDelegate {
            font.bold: true 
            implicitWidth: 200
            implicitHeight: 32
            anchors.leftMargin: 30 
            anchors.rightMargin: 30
            Text {
                id:sortIndicator
                anchors.verticalCenter: parent.verticalCenter
                anchors.right: parent.right
                color: "#3A3A3C"
                horizontalAlignment: Text.AlignRight
                text: " ▲" //+" ▼" 

            }        
        }
        model: root.headerLabels
    }

    TableView {
        id: tableView
        anchors.top: headerView.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        columnSpacing: 2
        // keyNavigationEnabled: true
        // pointerNavigationEnabled: true
        resizableColumns: true
        selectionBehavior: TableView.SelectRows
        selectionMode:TableView.SingleSelection
        editTriggers: TableView.NoEditTriggers
        selectionModel: ItemSelectionModel {}
        //property var columnWidths: root.columnWidths
        //rowHeight: 28
        clip: true

        model: TableModel {
            id: tableModel
            TableModelColumn { display: "name" }
            TableModelColumn { display: "type" }
            TableModelColumn { display: "size" }
            TableModelColumn { display: "modifed" }
            TableModelColumn { display: "path" }

            rows: [
                { "name": "Users2", "type":"Folder", "size":"4.3 MB", "modifed":"2024-06-01", "path":"/srs/" },
                { "name": "Users",  "type":"Folder", "size":"4 MB",   "modifed":"2024-06-11", "path":"/srs/" },
                { "name": "Users",  "type":"Folder", "size":"4 MB",   "modifed":"2024-06-11", "path":"/srs/rerert/ret/rert/ewtwe/wert/rewt" }
            ]
        }

        

        function sortBy(column) {
            var key = ["name","type","size","modifed","path"][column]
            if (!key) return
            var rows = tableModel.rows.slice()
            rows.sort(function(a,b){
                var av = a[key]
                var bv = b[key]
                return (""+av).localeCompare(""+bv)
            })
            tableModel.rows = rows
        }
        delegate: Rectangle {
            id: cellDelegate
            required property bool selected
            required property bool current
            required property int row
            required property int column
            
            implicitHeight: 22
            implicitWidth:[250,150,100,150,300][column]
            
            // Highlight entire row when selected
            color: cellDelegate.selected ? "#0051D5" :(row % 2 === 0 ? "#EFEFEF" : "#FAFAFA")
            

            Text {
                anchors.fill: parent
                anchors.leftMargin: 12
                anchors.rightMargin: 12
                text: {
                    if (column === 0) {
                        var icon = model.type === "Folder" ? "📁 " : "📄 "
                        return icon + display
                    }
                    return display
                }
                color: selected ? "#FFFFFF" : "#1C1C1E"
                elide: Text.ElideRight
                verticalAlignment: Text.AlignVCenter
                horizontalAlignment: column === 2 ? Text.AlignRight : Text.AlignLeft
            }

            MouseArea {
                anchors.fill: parent
                onClicked: {
                    tableView.selectionModel.setCurrentIndex(
                        tableView.model.index(row, 0),
                        ItemSelectionModel.ClearAndSelect | ItemSelectionModel.Rows
                    )
                }
                onDoubleClicked: {
                    if (model.type === "Folder") {
                        console.log("Navigate to:", model.path + model.name)
                    }
                }
            }
        }
        
        
        
        
        // TableViewDelegate {
        //     anchors.leftMargin: 20 
        //     anchors.rightMargin: 20
        //     implicitHeight: 28
        //     highlighted: selected
        //     TableView.editDelegate: null
            
        // }

        // delegate: Rectangle {
        //     implicitHeight: 32
        //     //color: styleData.selected ? "#0A84FF" : (styleData.row % 2 === 0 ? "#FFFFFF" : "#FAFAFA")
        //     border.color: "#E5E5EA"
        //     color: palette.base
        //     border.width: 1

        //     RowLayout {
        //         anchors.fill: parent
        //         anchors.leftMargin: 8
        //         anchors.rightMargin: 8
        //         spacing: 8

        //         Item {
        //             Layout.preferredWidth: tableView.columnWidthProvider(0)
        //             visible: styleData.column === 0
        //             RowLayout {
        //                 anchors.fill: parent
        //                 spacing: 6
        //                 Text {
        //                     text: styleData.rowData.type === "Folder" ? "📁" : "📄"
        //                     verticalAlignment: Text.AlignVCenter
        //                     font.pixelSize: 16
        //                 }
        //                 Text {
        //                     text: display
        //                     color: styleData.selected ? "#FFFFFF" : "#1C1C1E"
        //                     elide: Text.ElideRight
        //                     verticalAlignment: Text.AlignVCenter
        //                 }
        //             }
        //         }

        //         Text {
        //             visible: styleData.column !== 0
        //             text: styleData.value
        //             color: styleData.selected ? "#FFFFFF" : "#3A3A3C"
        //             elide: Text.ElideRight
        //             verticalAlignment: Text.AlignVCenter
        //             Layout.fillWidth: true
        //             horizontalAlignment: styleData.column === 2 ? Text.AlignRight : Text.AlignLeft
        //         }
        //     }

        //     MouseArea {
        //         anchors.fill: parent
        //         onDoubleClicked: {
        //             if (styleData.rowData.type === "Folder") {
        //                 // navigate into folder if wiring exists
        //             }
        //         }
        //     }
        // }
    }
}
