#!/usr/bin/env python

import sys
import os
import ctypes
import numpy as np
from PyQt5.QtCore import *
from PyQt5.QtGui import *
from PyQt5.QtWidgets import *

try:
    from dotenv import load_dotenv
    load_dotenv()
except ImportError as ex:
    print(f"module {ex.name} not used")

sequences = '''
   random
   faure05
   sobol
   sobol_rds
   sobol_owen
   sobol_owen_hash_lk
   sobol_owen_hash_v2
   sobol_owen_hash_fast
   sobol_owen_hash_good
'''.split()

genpoints = np.ctypeslib.load_library(os.getenv('GENPOINTS_LIB', 'genpoints'), '.')
array_1d_float32 = np.ctypeslib.ndpointer(dtype=np.float32, ndim=1, flags='CONTIGUOUS')
# extern "C" void genpoints(const char* seq, uint32_t n, uint32_t dim, uint32_t seed, float* x);
genpoints.genpoints.result = None
genpoints.genpoints.argtypes = [ctypes.c_char_p, ctypes.c_int32, ctypes.c_int32, ctypes.c_int32, array_1d_float32]
def zeros(nvals):
    return np.zeros(nvals, dtype=np.float32)

def genPoints(npoints, udim, vdim, seed, sequence):
    upoints = zeros(npoints)
    vpoints = zeros(npoints)
    if sequence == 'sobol_owen_decorrelated':
        sequence = 'sobol_owen'
        vseed = seed + 1
    else:
        vseed = seed
    genpoints.genpoints(bytes(sequence, encoding='utf-8'), npoints, udim, seed, upoints)
    genpoints.genpoints(bytes(sequence, encoding='utf-8'), npoints, vdim, vseed, vpoints)
    return np.column_stack((upoints, vpoints))

class Colors:
    points = QColor('black')
    bg = QColor('white')
    outline = QColor('gray')
    grid16 = QColor('lightGray')
    grid4 = QColor('gray')


def prev_power_of_two(n):
    if n < 1:
        return 0
    n |= n >> 1
    n |= n >> 2
    n |= n >> 4
    n |= n >> 8
    n |= n >> 16
    return (n + 1) >> 1


class PointView(QFrame):
    def __init__(self, parent=None):
        QFrame.__init__(self, parent)
        self.setMinimumSize(512, 512)
        self.setAutoFillBackground(True)
        palette = QPalette(self.palette())
        palette.setColor(QPalette.Window, Colors.bg)
        self.setPalette(palette)
        self.points = np.array([])
        self.showFFT = False

    def paintEvent(self, e):
        p = QPainter(self)
        self.paint(p)

    def paint(self, p):
        if self.showFFT:
            w = prev_power_of_two(min(self.width(), self.height()))
            ft = np.zeros( ( w, w ) )
            for x,y in self.points:
                if x >= 0 and x < 1 and y >= 0 and y < 1:
                    ft[int(x*w),int(y*w)] = 1
            ft -= ft.mean()
            ft = abs(np.fft.fftshift(np.fft.fft2(ft)))
            ft *= 64 / ft.mean()
            ft = ft.clip(0, 255).astype(np.uint32)
            ft = (255 << 24 | ft[:,:] << 16 | ft[:,:] << 8 | ft[:,:]).flatten()
            im = QImage(ft, 512, 512, QImage.Format_RGB32)

            rect = self.rect()
            scaled_size = im.size().scaled(rect.size(), Qt.AspectRatioMode.KeepAspectRatio)
            x = (rect.width() - scaled_size.width()) // 2
            y = (rect.height() - scaled_size.height()) // 2
            target_rect = QRect(x, y, scaled_size.width(), scaled_size.height())
            p.drawImage(target_rect, im)
        else:
            w = min(self.width(), self.height())
            s = w - 62
            p.translate((self.width() - w) / 2, (self.height() - w) / 2)
            s0, s1 = w/2-s/2,w/2+s/2
            p.setPen(Colors.outline)
            p.drawRect(int(s0), int(s0), int(s), int(s))
            s0 = (w-s)/2
            s1 = s0 + s

            if self.sequence.startswith('faure05'):
                for u in [i/25.0 for i in range(1,25)]:
                    if u in (.2, .4, .6, .8):
                        p.setPen(Colors.grid4)
                    else:
                        p.setPen(Colors.grid16)
                    su = s0 + u*s
                    p.drawLine(int(su), int(s0), int(su), int(s1))
                    p.drawLine(int(s0), int(su), int(s1), int(su))
            else:
                for u in [i/16.0 for i in range(1,16)]:
                    if u in (.25, .5, .75):
                        p.setPen(Colors.grid4)
                    else:
                        p.setPen(Colors.grid16)
                    su = s0 + u*s
                    p.drawLine(int(su), int(s0), int(su), int(s1))
                    p.drawLine(int(s0), int(su), int(s1), int(su))

            for x,y in self.points:
                p.setPen(QPen(Colors.points))
                p.drawPoint(int(s0-10), int((.5-y)*s+w/2))
                p.drawPoint(int((x-.5)*s+w/2), int(s0-10))

                pen = QPen(Colors.points)
                pen.setWidth(3)
                p.setPen(pen)
                p.drawPoint(int((x-.5)*s+w/2), int((.5-y)*s+w/2))

class Slider(QWidget):
    def __init__(self, parent, label, min, max, initial):
        QWidget.__init__(self, parent)
        self.value = None
        self.label = QLabel(label, self)
        self.slider = QSlider(Qt.Horizontal, self)
        self.spin = QSpinBox(self)
        self.spin.setMaximum(2**31 - 1)
        self.slider.valueChanged.connect(self.setValue)
        self.spin.valueChanged.connect(self.setValue)
        self.slider.setMinimum(min)
        self.slider.setMaximum(max)
        self.setValue(initial)
        l = QHBoxLayout(self)
        l.setContentsMargins(0,0,0,0)
        l.addWidget(self.label)
        l.addWidget(self.slider, stretch=1)
        l.addWidget(self.spin)
        self.label.setMinimumWidth(50)

    def setValue(self, val):
        try: val = int(val)
        except: return
        if self.value == val: return
        self.value = val
        self.slider.setValue(val)
        self.spin.setValue(val)

    def setMaximum(self, max):
        self.slider.setMaximum(max)

class Sampler(QWidget):
    def __init__(self, parent=None):
        QWidget.__init__(self, parent)
        self.pv = PointView(self)
        self.sequencetype_combobox = QComboBox(self)
        self.sequencetype_combobox.insertItems(0, sequences)

        self.n_slider = Slider(self, "nsamples", 1, 4096, 64)
        self.udim_slider = Slider(self, "u dim", 0, 7, 0)
        self.vdim_slider = Slider(self, "v dim", 0, 7, 1)
        self.seed_slider = Slider(self, "seed", 1, 100, 1)

        self.showFFT_checkbox = QCheckBox("show FFT", self)

        l = QVBoxLayout(self)
        l.setSpacing(5)
        l.addWidget(self.pv, stretch=1)

        combo_layout = QHBoxLayout(self)
        combo_layout.setSpacing(10)
        combo_layout.addWidget(QLabel("sequence", self), stretch=0)
        combo_layout.addWidget(self.sequencetype_combobox, stretch=1)
        combo_layout.addWidget(self.showFFT_checkbox, stretch=0)
        l.addLayout(combo_layout)

        slider_grid = QGridLayout(self)
        slider_grid.addWidget(self.n_slider, 0, 0)
        slider_grid.addWidget(self.udim_slider, 0, 1)
        slider_grid.addWidget(self.vdim_slider, 1, 0)
        slider_grid.addWidget(self.seed_slider, 1, 1)
        l.addLayout(slider_grid)

        self.sequencetype_combobox.activated.connect(self.updatePoints)
        self.n_slider.slider.valueChanged.connect(self.updatePoints)
        self.udim_slider.slider.valueChanged.connect(self.updatePoints)
        self.vdim_slider.slider.valueChanged.connect(self.updatePoints)
        self.seed_slider.slider.valueChanged.connect(self.updatePoints)
        self.showFFT_checkbox.stateChanged.connect(self.updatePoints)
        self.updatePoints()

    def updatePoints(self):
        npoints = self.n_slider.value
        udim = self.udim_slider.value
        vdim = self.vdim_slider.value
        seed = self.seed_slider.value
        sequence = self.sequencetype_combobox.currentText()
        self.pv.sequence = sequence
        self.pv.udim = udim
        self.pv.vdim = vdim
        self.pv.showFFT = self.showFFT_checkbox.checkState() == Qt.Checked
        self.pv.points = genPoints(npoints, udim, vdim, seed, sequence)
        self.pv.update()

app = QApplication(sys.argv)
w = Sampler()
w.show()
app.exec_()
