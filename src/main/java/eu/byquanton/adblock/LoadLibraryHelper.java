package eu.byquanton.adblock;

import eu.byquanton.adblock.exception.RustException;

import java.io.File;
import java.io.FileOutputStream;
import java.io.IOException;
import java.io.InputStream;

/**
 * Helper class to load native Rust libraries directly from JAR resources.
 */
class LoadLibraryHelper {

    /**
     * Public method to load the native library.
     *
     * @throws RustException if the library cannot be loaded.
     */
    public static void loadNativeLibrary() throws RustException {
        try {
            if (loadLibraryFromJavaLibPath()) {
                System.out.println("Loaded native library from java.library.path!");
                return;
            }

            String resourcePath = "/native/libadblock_coffee" + buildLibraryExtension();
            File tempLib = extractLibraryToTemp(resourcePath);
            System.load(tempLib.getAbsolutePath());
            System.out.println("Loaded native library from JAR resources: " + tempLib.getAbsolutePath());
        } catch (UnsatisfiedLinkError ex) {
            throw new RustException("Failed to load native library: " + ex.getMessage());
        }
    }

    /**
     * Attempts to load the library from java.library.path first.
     */
    private static boolean loadLibraryFromJavaLibPath() {
        try {
            System.loadLibrary("adblock_coffee");
            return true;
        } catch (UnsatisfiedLinkError ex) {
            return false;
        }
    }

    /**
     * Extracts a library resource from the JAR to a temporary file.
     *
     * @param resourcePath the path of the resource inside the JAR
     * @return File pointing to the temporary extracted file
     * @throws RustException if extraction fails
     */
    private static File extractLibraryToTemp(String resourcePath) throws RustException {
        try (InputStream is = LoadLibraryHelper.class.getResourceAsStream(resourcePath)) {
            if (is == null) {
                throw new RustException("Native library resource not found in JAR: " + resourcePath);
            }

            String suffix = buildLibraryExtension();
            File tempFile = File.createTempFile("libadblock_coffee_", suffix);
            tempFile.deleteOnExit();

            try (FileOutputStream os = new FileOutputStream(tempFile)) {
                byte[] buffer = new byte[8192];
                int read;
                while ((read = is.read(buffer)) != -1) {
                    os.write(buffer, 0, read);
                }
            }

            return tempFile;
        } catch (IOException e) {
            throw new RustException("Failed to extract native library: " + e.getMessage(), e);
        }
    }

    /**
     * This method generate extension by current system platform.
     *
     * @return String A native library extension by system platform.
     */
    private static String buildLibraryExtension() {
        String osName = System.getProperty("os.name").toLowerCase();
        if (osName.contains("windows")) {
            return ".dll";
        } else if (osName.contains("mac os x")) {
            return  ".dylib";
        } if (osName.contains("linux")) {
            return ".so";
        } else {
            System.err.println("Unsupported OS: " + osName);
            return ".so";
        }
    }
}
