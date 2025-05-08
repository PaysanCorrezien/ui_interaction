def run():
    # Get the currently focused window
    win = automation.focused_window()
    print(f"Focused window: {win.title}")

    # Try to find a control named "Login"
    login_button = automation.find_by_name("Login")
    if login_button:
        print("Found 'Login' control")
        login_button.click()
    else:
        print("No 'Login' control found")

    # Get the focused element in the current window
    focused = win.focused_element()
    if focused:
        print(f"Focused element: {focused.name} ({focused.control_type})")
        # Example of setting text if it's a text input
        if focused.control_type == "Edit":
            focused.set_text("Hello from Python!") 