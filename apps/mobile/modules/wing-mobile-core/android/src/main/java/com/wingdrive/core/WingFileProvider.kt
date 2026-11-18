package com.wingdrive.core

import androidx.core.content.FileProvider

/** A subclass of its own, so its manifest entry never clashes with another library's provider. */
class WingFileProvider : FileProvider()
