import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

ApplicationWindow {
    id: root

    visible: true
    title: "Sampler - " + sampler_backend.genpoints_lib_name

    minimumWidth: mainColumn.implicitWidth + 2 * mainColumn.anchors.margins
    minimumHeight: mainColumn.implicitHeight + 2 * mainColumn.anchors.margins

    function updatePoints() {
        sampler_backend.update_points(sequenceType.currentText, nSamples.value, uDim.value, vDim.value, seed.value);
    }

    Component.onCompleted: updatePoints()

    ColumnLayout {
        id: mainColumn

        spacing: 5
        anchors.fill: parent
        anchors.margins: 10

        Item {
            Layout.minimumWidth: 512
            Layout.minimumHeight: 512
            Layout.fillWidth: true
            Layout.fillHeight: true

            // color: "white"
            // border.color: "gray"

            Image {
                enabled: sampler_backend.show_fft
                visible: enabled
                cache: false
                fillMode: Image.PreserveAspectFit
                anchors.fill: parent
                source: sampler_backend.fft_image_url
                smooth: false
            }

            PointsView {
                enabled: !sampler_backend.show_fft
                visible: enabled
                anchors.fill: parent
            }
        }

        RowLayout {
            spacing: 10

            Label {
                text: "sequence"
            }

            ComboBox {
                id: sequenceType

                model: sampler_backend.sequences
                Layout.fillWidth: true
                onCurrentTextChanged: root.updatePoints()
            }

            CheckBox {
                id: showFFT

                text: "show FFT"
                checked: sampler_backend.show_fft
                onCheckedChanged: {
                    sampler_backend.show_fft = checked;
                    root.updatePoints();
                }
            }
        }

        Row {
            spacing: 10

            Label {
                text: "star discrepancy:"
            }
            Label {
                id: discrepancy
                text: sampler_backend.star_discrepancy.toFixed(3)
            }
        }

        GridLayout {
            columns: 2

            CompositeSlider {
                id: nSamples

                text: "nsamples"
                value: 64
                slider.from: 1
                slider.to: 4096
                onValueChanged: root.updatePoints()
            }

            CompositeSlider {
                id: uDim

                text: "u dim"
                slider.from: 0
                slider.to: 31
                value: 0
                onValueChanged: root.updatePoints()
            }

            CompositeSlider {
                id: vDim

                text: "v dim"
                slider.from: 0
                slider.to: 31
                value: 1
                onValueChanged: root.updatePoints()
            }

            CompositeSlider {
                id: seed

                text: "seed"
                slider.from: 1
                slider.to: 100
                value: 1
                onValueChanged: root.updatePoints()
            }
        }
    }
}
