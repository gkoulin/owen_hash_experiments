import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

RowLayout {
    id: root

    property int value

    property alias text: label.text

    property alias slider: slider
    property alias spinBox: spinBox

    onValueChanged: {
        slider.value = value;
        spinBox.value = value;
    }

    Label {
        id: label

        Layout.minimumWidth: 50
    }
    Slider {
        id: slider

        Layout.fillWidth: true

        onValueChanged: root.value = value
    }
    SpinBox {
        id: spinBox

        from: 0
        to: 2 ** 31 - 1
        stepSize: 1
        editable: true

        onValueChanged: root.value = value
    }
}
