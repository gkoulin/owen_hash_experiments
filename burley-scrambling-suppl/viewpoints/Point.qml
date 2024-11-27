import QtQuick
import QtQuick.Controls

Rectangle {
    id: point

    required property point position
    required property double scale
    required property int size
    property bool hovered: mouseArea.containsMouse

    x: position.x * scale - width / 2
    y: position.y * scale - height / 2

    width: hovered ? size * 2 : size
    height: width

    ToolTip.visible: hovered
    ToolTip.text: "(" + position.x.toFixed(3) + ", " + position.y.toFixed(3) + ")"

    MouseArea {
        id: mouseArea

        property int minHitSize: 10
        width: Math.max(parent.width, minHitSize)
        height: Math.max(parent.width, minHitSize)
        hoverEnabled: true
        anchors.centerIn: parent
    }
}
