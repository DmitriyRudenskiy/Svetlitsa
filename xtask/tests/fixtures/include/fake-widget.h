/* Фикстура парити: публичный заголовок «как в MATE». Ровно одна публичная функция —
 * fake_widget_new. Всё остальное — макросы, typedef-ы, static и члены-указатели —
 * генератор обязан проглотить. */

#ifndef FAKE_WIDGET_H
#define FAKE_WIDGET_H

#include <glib-object.h>

G_BEGIN_DECLS

#define FAKE_TYPE_WIDGET (fake_widget_get_type ())
G_DECLARE_FINAL_TYPE (FakeWidget, fake_widget, FAKE, WIDGET, GtkButton)

#define FAKE_AVAILABLE(f, m, n) __attribute__((availability(fake,f,m,n)))

typedef struct _FakeWidgetPrivate FakeWidgetPrivate;
typedef void (*FakeChangedFunc) (FakeWidget *widget, gpointer data);

struct _FakeWidget {
  GtkButton parent_instance;
  void (*changed) (FakeWidget *widget);
};

/**
 * fake_widget_new:
 * @title: заголовок
 * @width: ширина (прототип нарочно многострочный)
 *
 * Возвращает: новый виджет.
 */
FAKE_AVAILABLE (2,0)
FakeWidget *fake_widget_new (const char *title,
                             int width);

static inline void fake_helper_unused (void) {}

G_END_DECLS

#endif /* FAKE_WIDGET_H */
