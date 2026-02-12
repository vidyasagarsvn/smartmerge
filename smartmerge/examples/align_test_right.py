# Alignment test - RIGHT


def common_header():
    title = "SmartMerge"
    version = "1.0"
    return f"{title} v{version}"


def right_only():
    return "right"


class Widget:
    def __init__(self, name):
        self.name = name

    def render(self):
        return f"Widget({self.name})"


# identical footer block
if __name__ == "__main__":
    print(common_header())
    print(Widget("alpha").render())
