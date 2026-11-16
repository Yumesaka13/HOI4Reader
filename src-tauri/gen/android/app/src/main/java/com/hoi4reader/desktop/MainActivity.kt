package com.hoi4reader.desktop

import android.os.Bundle
import androidx.activity.SystemBarStyle
import androidx.activity.enableEdgeToEdge
import androidx.core.view.ViewCompat
import androidx.core.view.WindowInsetsCompat
import androidx.core.view.updatePadding

class MainActivity : TauriActivity() {
  override fun onCreate(savedInstanceState: Bundle?) {
    val bars = SystemBarStyle.dark(0xff18161c.toInt())
    enableEdgeToEdge(statusBarStyle = bars, navigationBarStyle = bars)
    super.onCreate(savedInstanceState)
    val content = findViewById<android.view.View>(android.R.id.content)
    content.setBackgroundColor(0xff18161c.toInt())
    ViewCompat.setOnApplyWindowInsetsListener(content) { view, insets ->
      val safe = insets.getInsets(
        WindowInsetsCompat.Type.systemBars() or WindowInsetsCompat.Type.displayCutout()
      )
      view.updatePadding(left = safe.left, top = safe.top, right = safe.right, bottom = safe.bottom)
      insets
    }
    ViewCompat.requestApplyInsets(content)
  }
}
