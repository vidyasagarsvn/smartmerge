# Alignment test - LEFT


def common_header():
    title = "SmartMerge"
    version = "1.0"
    return f"{title} v{version}"


def left_only():
    return "left"


def compute(a, b):
    result = a + b
    return result


class Widget:
    def __init__(self, name):
        self.name = name

    def render(self):
        return f"Widget({self.name})"


# identical footer block
if __name__ == "__main__":
    print(common_header())
    print(Widget("alpha").render())
