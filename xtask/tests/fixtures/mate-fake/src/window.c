/* Фикстура: маркеры X11 нужны тесту раздела «X-механизмы оригинала» (ADR-0016). */
#include <X11/Xlib.h>
int main(void){ XOpenDisplay(0); XGrabKey(0,1,2,3,0,0); return 0; }
