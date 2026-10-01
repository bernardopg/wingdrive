import XCTest
@testable import WingDriveClient

final class WingDriveClientTests: XCTestCase {

    func testClientInitialization() {
        let client = WingDriveClient(socketPath: "/tmp/test.sock")
        XCTAssertNotNil(client)
    }

    func testErrorTypes() {
        let error = WingDriveError.connectionFailed("Test error")
        XCTAssertNotNil(error.errorDescription)
        XCTAssertTrue(error.errorDescription!.contains("Test error"))
    }

    // More tests will be added once the daemon connection is implemented
}
