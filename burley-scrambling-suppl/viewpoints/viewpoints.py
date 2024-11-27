#!/usr/bin/env python

import sys
import os
from pathlib import Path
import ctypes
import numpy as np
from PySide6.QtCore import Qt, QObject, Slot, Signal, Property, QPointF
from PySide6.QtGui import QColor, QGuiApplication, QImage
from PySide6.QtQuick import QQuickImageProvider
from PySide6.QtQml import QQmlApplicationEngine

import signal

# Allow programme to be killed with Ctrl-C
signal.signal(signal.SIGINT, signal.SIG_DFL)

try:
    from dotenv import load_dotenv

    load_dotenv()
except ImportError as ex:
    print(f"module {ex.name} not used")


class SamplerBackend(QObject):
    def __init__(self):
        super().__init__()
        self._show_fft = False
        self._genpoints = np.ctypeslib.load_library(os.getenv('GENPOINTS_LIB', 'genpoints'), '.')
        array_1d_float32 = np.ctypeslib.ndpointer(dtype=np.float32, ndim=1, flags='CONTIGUOUS')

        # extern "C" void genpoints(const char* seq, uint32_t n, uint32_t dim, uint32_t seed, float* x);
        self._genpoints.genpoints.restype = None
        self._genpoints.genpoints.argtypes = [
            ctypes.c_char_p,
            ctypes.c_int32,
            ctypes.c_int32,
            ctypes.c_int32,
            array_1d_float32,
        ]

        self._genpoints.sequence_names.restype = ctypes.c_char_p
        self._genpoints.sequence_names.argtypes = []
        self._sequences = self._genpoints.sequence_names().decode('utf-8').split(";")
        self.sequences_changed.emit()
        self._fft_image_url = ""

        if hasattr(self._genpoints, "init"):
            self._genpoints.init()

        self._points = np.array([])
        self._star_discrepancy = 0

    sequences_changed = Signal()

    @Property(list, notify=sequences_changed)  # type: ignore
    def sequences(self):
        return self._sequences

    show_fft_changed = Signal()

    def get_show_fft(self):
        return self._show_fft

    def set_show_fft(self, value):
        if self._show_fft != value:
            self._show_fft = value
            self.show_fft_changed.emit()

    show_fft = Property(bool, get_show_fft, set_show_fft, notify=show_fft_changed)  # type: ignore

    points_changed = Signal()

    @Property(str, notify=points_changed)  # type: ignore
    def fft_image_url(self):
        return "image://fft_image/" + self._fft_image_url

    @Slot(str, int, int, int, int)
    def update_points(self, sequence: str, npoints: int, udim: int, vdim: int, seed: int):
        self._fft_image_url = f"{sequence}-{npoints}-{udim}-{vdim}-{seed}"
        self._points = self.gen_points(sequence, npoints, udim, vdim, seed)
        self._star_discrepancy = star_discrepancy(self._points)
        self.points_changed.emit()

    def gen_points(self, sequence: str, npoints: int, udim: int, vdim: int, seed: int):
        def zeros(nvals):
            return np.zeros(nvals, dtype=np.float32)

        upoints = zeros(npoints)
        vpoints = zeros(npoints)
        self._genpoints.genpoints(bytes(sequence, encoding='utf-8'), npoints, udim, seed, upoints)
        self._genpoints.genpoints(bytes(sequence, encoding='utf-8'), npoints, vdim, seed, vpoints)
        return np.column_stack((upoints, vpoints))

    @Property(float, notify=points_changed)  # type: ignore
    def star_discrepancy(self):
        return self._star_discrepancy

    @Property(list, notify=points_changed)  # type: ignore
    def points(self):
        return [QPointF(row[0], row[1]) for row in self._points]

    @Property(str, constant=True)
    def genpoints_lib_name(self):
        return Path(self._genpoints._name).name


class FFTImageProvider(QQuickImageProvider):
    def __init__(self, sampler_backend: SamplerBackend):
        super().__init__(QQuickImageProvider.ImageType.Image)
        self.sampler_backend = sampler_backend

    def requestImage(self, id, size, requestedSize):
        points = self.sampler_backend._points
        if len(points) == 0:
            return QImage()

        w = prev_power_of_two(min(size.width(), size.height()))
        if w <= 0:
            w = 512

        ft = np.zeros((w, w))

        for x, y in points:
            if x >= 0 and x < 1 and y >= 0 and y < 1:
                ft[int(x * w), int(y * w)] = 1
        ft -= ft.mean()
        ft = abs(np.fft.fftshift(np.fft.fft2(ft)))
        ft *= 64 / ft.mean()
        ft = ft.clip(0, 255).astype(np.uint32)
        ft = (255 << 24 | ft[:, :] << 16 | ft[:, :] << 8 | ft[:, :]).flatten()
        im = QImage(ft, w, w, QImage.Format.Format_RGB32)
        return im


def prev_power_of_two(n):
    if n < 1:
        return 0
    n |= n >> 1
    n |= n >> 2
    n |= n >> 4
    n |= n >> 8
    n |= n >> 16
    return (n + 1) >> 1


def star_discrepancy(points):
    n, _ = points.shape
    if n == 0:
        return 0
    volumes = np.prod(points, axis=1)
    counts = np.sum(np.all(np.swapaxes(points[:, np.newaxis, :] <= points, 0, 1), axis=2), axis=1)
    discrepancies = np.abs(counts / n - volumes)
    max_discrepancy = np.max(discrepancies)
    return max_discrepancy


def main():
    app = QGuiApplication(sys.argv)
    engine = QQmlApplicationEngine()

    sampler_backend = SamplerBackend()
    engine.rootContext().setContextProperty("sampler_backend", sampler_backend)
    engine.addImageProvider("fft_image", FFTImageProvider(sampler_backend))
    engine.load(Path(__file__).resolve().parent / "sampler.qml")
    if not engine.rootObjects():
        sys.exit(-1)
    sys.exit(app.exec())


if __name__ == "__main__":
    main()
