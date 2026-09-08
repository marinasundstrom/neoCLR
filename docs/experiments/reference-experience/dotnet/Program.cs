
var products = new List<Product> { new("Coffee", 12, 5), new("Tea", 8, 3) };
var notifications = new ConsoleNotifications();
var receipts = new List<Receipt>();
var product = products.Find(product => product.Name == "Coffee")
    ?? throw new Exception("Missing product");
var restock = MakeRestock(product);
var outcome = Purchase(product, 2, notifications);
if (!outcome.Accepted) throw new Exception(outcome.Message);
receipts.Add(outcome.Receipt);
product.Price = 15;
restock(4);
Console.WriteLine(product.Stock);
Console.WriteLine(receipts[0].Total);
var rejected = Purchase(product, 99, notifications);
if (rejected.Accepted || product.Stock != 7 || receipts[0].Total != 24)
    throw new Exception("Incorrect order state");
Console.WriteLine(rejected.Message);
if (products.Find(product => product.Name == "Milk") is not null)
    throw new Exception("Unexpected product");
Console.WriteLine("Orders complete");

// Same capacity and edits as list-copy.neo; assignment aliases the entire List.
var original = new List<int>(2) { 10 };
var copy = original;
copy[0] = 20;
Console.WriteLine(original[0]);
copy.Add(30);
Console.WriteLine(original.Count);
Console.WriteLine(copy.Count);
copy.Add(40);
copy[0] = 50;
Console.WriteLine(original[0]);
Console.WriteLine(copy[0]);
if (original.Count != 3 || original[0] != 50) throw new Exception("List alias failed");

static Action<int> MakeRestock(Product product) => amount => product.Stock += amount;
static PurchaseOutcome Purchase(Product product, int quantity, Notifications notifications)
{
    if (quantity <= 0) return new(false, default, "Invalid quantity");
    if (product.Stock < quantity) return new(false, default, "Out of stock");
    product.Stock -= quantity;
    var receipt = new Receipt(product.Name, quantity, product.Price * quantity);
    notifications.Purchased(receipt);
    return new(true, receipt, "");
}
sealed class Product(string name, int price, int stock)
{
    public string Name = name;
    public int Price = price;
    public int Stock = stock;
}
readonly record struct Receipt(string Name, int Quantity, int Total);
readonly record struct PurchaseOutcome(bool Accepted, Receipt Receipt, string Message);
interface Notifications { void Purchased(Receipt receipt); }
sealed class ConsoleNotifications : Notifications
{
    public void Purchased(Receipt receipt)
    {
        Console.WriteLine("Purchased: " + receipt.Name);
        Console.WriteLine(receipt.Total);
    }
}
