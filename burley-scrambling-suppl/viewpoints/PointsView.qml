pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Shapes

Item {
    id: root

    property int pointSize: 3
    property color pointColor: "black"

    Rectangle {
        id: background

        color: "white"
        anchors.centerIn: parent
        width: Math.min(parent.width, parent.height)
        height: width
    }

    GridLayout {
        id: grid
        anchors.fill: background
        anchors.margins: 10

        columns: 2

        Item {}

        Item {
            id: xProjection

            implicitHeight: 5
            Layout.fillWidth: true

            transform: Scale {
                origin.y: xProjection.height / 2
                yScale: -1
            }

            Repeater {
                model: sampler_backend.points

                delegate: Point {
                    required property point modelData
                    position: modelData
                    y: (xProjection.height - height) / 2
                    color: root.pointColor
                    scale: xProjection.width
                    size: root.pointSize
                    opacity: hovered ? 1.0 : 0.1
                }
            }
        }

        Item {
            id: yProjection

            implicitWidth: 5
            Layout.fillHeight: true

            transform: Scale {
                origin.y: yProjection.height / 2
                yScale: -1
            }

            Repeater {
                model: sampler_backend.points

                delegate: Point {
                    required property point modelData
                    position: modelData
                    x: (yProjection.width - width) / 2
                    color: root.pointColor
                    scale: yProjection.height
                    size: root.pointSize
                    opacity: hovered ? 1.0 : 0.1
                }
            }
        }

        Item {
            id: canvas

            Layout.fillHeight: true
            Layout.fillWidth: true

            transform: Scale {
                origin.y: canvas.height / 2
                yScale: -1
            }

            property double scale: width

            Canvas {
                id: gridLines

                anchors.fill: parent
                onPaint: {
                    var ctx = getContext("2d");
                    strokeGrid(ctx, "lightGray", 16);
                    strokeGrid(ctx, "gray", 4);
                }

                function strokeGrid(ctx, color, n) {
                    ctx.strokeStyle = color;
                    ctx.beginPath();
                    for (var i = 0; i < n; ++i) {
                        var pos = i / n;
                        ctx.moveTo(pos * canvas.scale, 0);
                        ctx.lineTo(pos * canvas.scale, canvas.height);
                        ctx.moveTo(0, pos * canvas.scale);
                        ctx.lineTo(width, pos * canvas.scale);
                    }
                    ctx.stroke(); // Draw the line
                }
            }

            Rectangle {
                anchors.fill: parent
                color: "transparent"
                border.color: "black"
            }

            Repeater {
                model: sampler_backend.points

                delegate: Point {
                    required property point modelData
                    position: modelData
                    color: root.pointColor
                    scale: canvas.scale
                    size: root.pointSize
                }
            }
        }
    }
}
