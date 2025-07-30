fun promptForNumber(): Int {
    print("Enter a starting number: ")
    var number: Int? = readlnOrNull()?.toIntOrNull()
    while (number == null) {
        println("Invalid number, please try again.")
        number = readlnOrNull()?.toIntOrNull()
    }

    return number
}
